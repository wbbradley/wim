use crate::editor::{Editor, HandleKey};
use crate::plugin::{load_plugin, PluginRef};
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
    let plugin = load_plugin()?;

    let termios = Arc::new(Termios::enter_raw_mode());
    let panic_termios = termios.clone();
    std::panic::set_hook(Box::new(move |p| {
        panic_termios.exit_raw_mode();
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
            editor.display(&mut buf, &editor);
            should_refresh = false;
        }
        let key = if let Some(key) = read_key() {
            key_timeout = Some(Instant::now() + Duration::from_secs(1));
            key
        } else if let Some(next_key_timeout) = key_timeout {
            if Instant::now() > next_key_timeout {
                key_timeout = None;
                Key::None
            } else {
                continue;
            }
        } else {
            continue;
        };
        assert!(dks.is_empty());
        dks.push_front(DK::Key(key));
        should_refresh = true;
        pump(&mut editor, &mut dks)?;
    }
    Ok(())
}
fn pump(editor: &mut Editor, dks: &mut VecDeque<DK>) -> anyhow::Result<()> {
    loop {
        match editor.handle_keys(dks) {
            HandleKey::DK(dk) => match dk {
                DK::Key(key) => {
                    panic!(
                        "Keys should be translated before they propagate! [key={}]",
                        key
                    );
                }
                DK::Dispatch(view_key, message) => {
                    return match message {
                        Message::SendKey(key) => {
                            trace!("[dk loop] send_key({:?}, {:?})...", view_key, key);
                            editor
                                .send_key_to_view(view_key, key)
                                .context("send_key_to_view")
                        }
                        Message::Command { name, args } => match editor.execute_command(name, args)
                        {
                            Ok(status) => {
                                editor.set_status(status);
                                Ok(())
                            }
                            Err(error) => {
                                trace!("error: {}", error);
                                Err(error).context("DK::Command")
                            }
                        },
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
                editor.set_status(status!("...{:?}", choices));
            }
        }
    }
}
