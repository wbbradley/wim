use crate::editor::{Editor, HandleKey};
use crate::plugin::{Plugin, PluginRef};
use crate::prelude::*;
use crate::read::read_key;
use crate::termios::Termios;
use crate::types::Rect;
use anyhow::Context as AnyhowContext;
use log::LevelFilter;
use std::env;

mod bindings;
mod buf;
mod command;
mod commandline;
mod consts;
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
mod propvalue;
mod read;
mod rel;
mod row;
mod status;
mod termios;
mod trie;
mod types;
mod utils;
mod view;
mod vstack;
mod widechar_width;

pub static VERSION: &str = "v0.1.0";

fn main() -> anyhow::Result<()> {
    let plugin = Plugin::new();

    let termios = Arc::new(Termios::enter_raw_mode());
    let panic_termios = termios.clone();
    std::panic::set_hook(Box::new(move |p| {
        panic_termios.exit_raw_mode();
        log::error!("{}", std::backtrace::Backtrace::force_capture());
        log::error!("{}", p);
        println!("{}", p);
    }));

    let res = run_app(plugin);
    termios.exit_raw_mode();
    res
}

fn run_app(plugin: PluginRef) -> anyhow::Result<()> {
    simple_logging::log_to_file("wim.log", LevelFilter::Trace)?;
    let args: Vec<String> = env::args().collect();
    trace!("wim run with args: {:?}", args);

    let frame: Rect = Termios::get_window_size().into();

    let mut editor = Editor::new(plugin);
    if args.len() > 1 {
        editor.execute_command("open".to_string(), vec![CallArg::String(args[1].clone())])?;
    }
    let mut buf = Buf::default();
    let mut should_refresh = true;
    let mut dks: VecDeque<DK> = Default::default();
    let mut key_timeout: Option<Instant> = None;

    while !editor.get_should_quit() {
        if should_refresh {
            editor.layout(frame);
            buf.truncate();
            trace!("redisplaying!");
            editor.display(&mut buf, &editor);
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
        pump(&mut editor, &mut dks)?;
    }
    Ok(())
}
fn pump(editor: &mut Editor, dks: &mut VecDeque<DK>) -> anyhow::Result<()> {
    while matches!(dks.front(), Some(DK::Key(Key::None))) {
        trace!("popping Key::None off dks");
        dks.pop_front();
    }
    if dks.is_empty() {
        return Ok(());
    }
    loop {
        match editor.handle_keys(dks) {
            HandleKey::DK(dk) => match dk {
                DK::Key(_) => {
                    dks.push_front(dk);
                    continue;
                }
                DK::Dispatch(view_key, message) => {
                    let view = editor.get_view_or_focused_view_mut(view_key);
                    let result = match message {
                        Message::SendKey(key) => view.send_key(key),
                        Message::Command { name, args } => view.execute_command(name, args),
                    };
                    editor
                        .eat_status_result(result)
                        .context("execute_command")?;
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
