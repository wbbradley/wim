use crate::editor::Editor;
use crate::error::{ErrorContext, Result};
use crate::layout::recursive_layout;
use crate::plugin::{Plugin, PluginRef};
use crate::prelude::*;
use crate::read::read_key;
use crate::termios::Termios;
use crate::types::Rect;
use crate::view_map::HandleKey;
use log::LevelFilter;
use signal_hook::consts::signal::SIGWINCH;
use signal_hook::flag as signal_flag;
use std::env;
use std::fmt::Write;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod bindings;
mod bitmap;
mod buf;
mod classify;
mod color;
mod command;
mod commandline;
mod consts;
mod dispatch;
mod dk;
mod doc;
mod docview;
mod editor;
mod error;
mod files;
mod format;
mod glyph;
mod key;
mod keygen;
mod layout;
mod message;
mod noun;
mod plugin;
mod prelude;
mod read;
mod rel;
mod row;
mod size;
mod status;
mod target;
mod termios;
mod trie;
mod types;
mod utils;
mod variant;
mod view;
mod view_map;
mod viewref;
mod vstack;
mod widechar_width;

pub static VERSION: &str = "v0.1.0";

fn main() -> Result<()> {
    simple_logging::log_to_file("wim.log", LevelFilter::Trace)?;
    let plugin = Plugin::new();

    let termios = Arc::new(Termios::enter_raw_mode());
    let panic_termios = termios.clone();
    std::panic::set_hook(Box::new(move |p| {
        panic_termios.exit_raw_mode();
        log::error!("{}", std::backtrace::Backtrace::force_capture());
        log::error!("{}", p);
        println!("{}", p);
    }));

    let view_map: crate::view_map::ViewMap = ViewMap::new();
    let res = run_app(plugin, view_map);
    termios.exit_raw_mode();
    res
}

fn write_bmp_diff(
    buf: &mut Buf,
    bmp_last: &mut Bitmap,
    bmp: &mut Bitmap,
    fd: libc::c_int,
) -> Result<()> {
    buf.truncate(0);
    buf.extend(b"\x1b[?25l");
    Bitmap::diff(bmp_last, bmp, buf)?;
    if let Some(cursor) = bmp.get_cursor() {
        write!(buf, "\x1b[{};{}H", cursor.y + 1, cursor.x + 1).context("write-cursor")?
    }
    buf.extend(b"\x1b[?25h");
    let ret = unsafe { libc::write(fd, buf.as_ptr() as *const libc::c_void, buf.len()) };
    if ret == -1 {
        crate::utils::die!("failed when calling libc::write");
    }
    assert!(ret == buf.len() as isize);
    std::mem::swap(bmp, bmp_last);
    Ok(())
}

fn run_app(plugin: PluginRef, mut view_map: ViewMap) -> Result<()> {
    let args: Vec<String> = env::args().collect();
    trace!("wim run with args: {:?}", args);

    let editor_view_key = Editor::install(plugin, &mut view_map);
    let mut editor: ViewRef = view_map.get_view(editor_view_key);
    let mut should_refresh = true;
    let should_resize = Arc::new(AtomicBool::new(false));
    signal_flag::register(SIGWINCH, Arc::clone(&should_resize))?;

    let default_glyph = Glyph {
        ch: ' ',
        format: BgColor::Rgb {
            r: 60,
            g: 50,
            b: 40,
        }
        .into(),
    };
    let mut dks: VecDeque<DK> = Default::default();
    let mut key_timeout: Option<Instant> = None;
    if args.len() > 1 {
        let filename = args[1].as_ref();
        dks.push_back(command("open").arg(filename).at_focused());
    }
    let mut layout_rects: HashMap<ViewKey, Rect> = Default::default();
    let mut terminal_size: Size = Termios::get_window_size();
    let mut bmp = Bitmap::new(terminal_size, default_glyph);
    let mut bmp_last = bmp.clone();

    let mut buf = Buf::default();
    while !editor.get_property_bool(crate::consts::PROP_EDITOR_SHOULD_QUIT, true) {
        if should_resize.swap(false, Ordering::Relaxed) {
            should_refresh = true;
            terminal_size = Termios::get_window_size();
            bmp.resize(terminal_size);
            bmp_last.resize_with_junk(terminal_size);
        }

        if should_refresh {
            layout_rects.clear();
            recursive_layout(
                &view_map,
                view_map.get_root_view_key(),
                terminal_size.into(),
                &mut layout_rects,
            );

            // Render the composite bitmap.
            bmp.clear();
            for (&vk, &frame) in layout_rects.iter() {
                let mut bmp_view = BitmapView::new(&mut bmp, frame);
                view_map.get_view(vk).display(&view_map, &mut bmp_view);
            }
            // Rasterize the bitmap to the terminal and swap the write buffers..
            write_bmp_diff(&mut buf, &mut bmp_last, &mut bmp, libc::STDIN_FILENO)?;

            should_refresh = false;
        }
        if matches!(dks.front(), Some(DK::Key(_)) | None) {
            if let Some(key) = read_key() {
                trace!("read key '{:?}'", key);
                key_timeout = Some(Instant::now() + Duration::from_secs(1));
                dks.push_back(DK::Key(key));
            } else if let Some(next_key_timeout) = key_timeout {
                if dks.front().is_none() {
                    // We're not waiting for any completion.
                    key_timeout = None;
                    continue;
                } else {
                    // We're waiting for a key completion.
                    if Instant::now() > next_key_timeout {
                        key_timeout = None;
                        dks.push_back(DK::Key(Key::None));
                    } else {
                        continue;
                    }
                }
            } else {
                continue;
            };
        }
        should_refresh = true;
        pump(&mut view_map, &mut dks, &mut editor)?;
    }
    Ok(())
}
fn pump(view_map: &mut ViewMap, dks: &mut VecDeque<DK>, editor: &mut ViewRef) -> Result<()> {
    while matches!(dks.front(), Some(DK::Key(Key::None))) {
        trace!("popping Key::None off dks");
        dks.pop_front();
    }
    loop {
        if dks.is_empty() {
            return Ok(());
        }
        match view_map.handle_keys(dks) {
            HandleKey::DK(dk) => match dk {
                DK::Key(_) => {
                    dks.push_front(dk);
                    continue;
                }
                DK::Dispatch(target, message) => {
                    let mut dispatch_target = view_map.resolve(target);
                    let result = match message {
                        Message::SendKey(key) => dispatch_target.send_key(key),
                        Message::Command { name, args } => {
                            dispatch_target.execute_command(name, args)
                        }
                    };
                    editor.set_status(result?);
                }
                DK::Sequence(next_dks) => {
                    next_dks
                        .iter()
                        .rev()
                        .cloned()
                        .for_each(|dk| dks.push_front(dk));
                }
            },
            HandleKey::Choices(choices) => {
                editor.set_status(status!("Valid next bindings: {:?}", choices));
                return Ok(());
            }
        }
    }
}
