use crate::editor::Editor;
use crate::plugin::{Plugin, PluginRef};
use crate::prelude::*;
use crate::read::read_key;
use crate::termios::Termios;
use crate::types::Rect;
use crate::view_map::HandleKey;
use log::LevelFilter;
use std::env;

mod bindings;
mod buf;
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
mod key;
mod keygen;
mod line;
mod mode;
mod noun;
mod plugin;
mod prelude;
mod read;
mod rel;
mod row;
mod status;
mod termios;
mod trie;
mod types;
mod utils;
mod variant;
mod view;
mod view_map;
mod vstack;
mod widechar_width;

pub static VERSION: &str = "v0.1.0";

fn main() -> anyhow::Result<()> {
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

fn run_app(plugin: PluginRef, view_map: ViewMap) -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    trace!("wim run with args: {:?}", args);

    let frame: Rect = Termios::get_window_size().into();

    let editor_view_key = Editor::install(plugin, &mut view_map);
    if args.len() > 1 {
        let editor: &mut dyn View = view_map.get_view_mut(editor_view_key);
        editor.execute_command("open".to_string(), vec![Variant::String(args[1].clone())])?;
    }
    let mut buf = Buf::default();
    let mut should_refresh = true;
    let mut dks: VecDeque<DK> = Default::default();
    let mut key_timeout: Option<Instant> = None;
    while !view_map
        .get_view(editor_view_key)
        .as_view_context()
        .get_property_bool(crate::consts::PROP_EDITOR_SHOULD_QUIT, true)
    {
        if should_refresh {
            let editor: &mut dyn View = view_map.get_view_mut(editor_view_key);
            editor.layout(&mut view_map, frame);
            buf.truncate();
            trace!("redisplaying!");
            editor.display(&view_map, &mut buf, &view_map);
            should_refresh = false;
        }
        if matches!(dks.front(), Some(DK::Key(_)) | None) {
            if let Some(key) = read_key() {
                key_timeout = Some(Instant::now() + Duration::from_secs(1));
                dks.push_back(DK::Key(key));
            } else if let Some(next_key_timeout) = key_timeout {
                if Instant::now() > next_key_timeout {
                    key_timeout = None;
                    dks.push_back(DK::Key(Key::None));
                } else {
                    continue;
                }
            } else {
                continue;
            };
        }
        should_refresh = true;
        pump(&mut view_map, &mut dks, editor_view_key)?;
    }
    Ok(())
}
fn pump(
    view_map: &mut ViewMap,
    dks: &mut VecDeque<DK>,
    editor_view_key: ViewKey,
) -> anyhow::Result<()> {
    while matches!(dks.front(), Some(DK::Key(Key::None))) {
        trace!("popping Key::None off dks");
        dks.pop_front();
    }
    if dks.is_empty() {
        return Ok(());
    }
    loop {
        match view_map.handle_keys(dks) {
            HandleKey::DK(dk) => match dk {
                DK::Key(_) => {
                    dks.push_front(dk);
                    continue;
                }
                DK::Dispatch(target, message) => {
                    let dispatch_target: &mut dyn DispatchTarget = view_map.resolve_mut(target);
                    let result = match message {
                        Message::SendKey(key) => dispatch_target.send_key(key),
                        Message::Command { name, args } => {
                            dispatch_target.execute_command(name, args)
                        }
                    };
                    match result {
                        Ok(status) => {
                            let editor: &mut dyn View = view_map.get_view_mut(editor_view_key);
                            editor.set_status(status);
                        }
                        Err(error) => anyhow::bail!(error),
                    }
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
                let editor: &mut dyn View = view_map.get_view_mut(editor_view_key);
                editor.set_status(status!("Valid next bindings: {:?}", choices));
                return Ok(());
            }
        }
    }
}
