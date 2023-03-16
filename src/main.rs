use crate::buf::Buf;
use crate::command::Command;
use crate::dk::DK;
use crate::editor::Editor;
use crate::key::Key;
use crate::plugin::{load_plugin, PluginRef};
use crate::read::read_key;
use crate::termios::Termios;
use crate::types::Rect;
use crate::view::View;
use anyhow::Context as AnyhowContext;
use log::LevelFilter;
use std::collections::VecDeque;
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
mod propvalue;
mod read;
mod rel;
mod row;
mod status;
mod termios;
mod trigger;
mod types;
mod utils;
mod view;
mod vstack;
mod widechar_width;

pub static VERSION: &str = "v0.1.0";

fn main() -> anyhow::Result<()> {
    let termios = Termios::enter_raw_mode();

    std::panic::set_hook(Box::new(move |p| {
        termios.exit_raw_mode();
        log::error!("{}", p);
        println!("{}", p);
    }));

    let plugin = load_plugin()?;
    // UnwindSafe
    run_app(plugin)
}

fn run_app(plugin: PluginRef) -> anyhow::Result<()> {
    simple_logging::log_to_file("wim.log", LevelFilter::Trace)?;
    let args: Vec<String> = env::args().collect();
    log::trace!("wim run with args: {:?}", args);

    let frame: Rect = Termios::get_window_size().into();

    let mut edit = Editor::new(plugin);
    if args.len() > 1 {
        edit.execute_command(Command::Open {
            filename: args[1].clone(),
        })?;
    }
    let mut buf = Buf::default();
    let mut should_refresh = true;
    let mut keys: VecDeque<Key> = Default::default();
    let mut dks: VecDeque<DK> = Default::default();
    let mut last_dk = DK::Noop;

    'outer: loop {
        if should_refresh {
            edit.layout(frame);
            buf.truncate();
            edit.display(&mut buf, &edit);
            should_refresh = false;
        }
        let key = match keys.pop_front() {
            Some(key) => key,
            None => {
                if let Some(key) = read_key() {
                    key
                } else {
                    continue;
                }
            }
        };
        assert!(dks.is_empty());
        dks.push_front(DK::Key(key));
        while let Some(mut dk) = dks.pop_front() {
            log::trace!("handling dk: {:?}", dk);
            if let DK::Trie { choices } = &last_dk {
                if let DK::Key(key) = dk {
                    if let Some(chosen_dk) = choices.get(&key) {
                        dks.push_front(dk);
                        dk = chosen_dk.clone();
                    } else if let Some(chosen_dk) = choices.get(&Key::None) {
                        /* user pressed something else but we have a mapping here. Use it. */
                        dks.push_front(dk);
                        dk = chosen_dk.clone();
                    }
                } else if let Some(chosen_dk) = choices.get(&Key::None) {
                    /* something else is queued but we need to run this dk */
                    dks.push_front(dk);
                    dk = chosen_dk.clone();
                } else {
                    edit.set_status(crate::status::Status::Message {
                        message: "Dropped key sequence.".into(),
                        expiry: std::time::Instant::now() + std::time::Duration::from_secs(1),
                    });
                }
            }
            last_dk = DK::Noop;
            match dk {
                DK::Trie { .. } => {
                    last_dk = dk;
                    continue;
                }
                DK::Key(key) => {
                    std::mem::drop(dk);
                    let next_dk = edit.handle_key(key)?;
                    dks.push_front(next_dk);
                    should_refresh = true;
                }
                DK::Sequence(next_dks) => {
                    next_dks
                        .iter()
                        .rev()
                        .cloned()
                        .for_each(|dk| dks.push_front(dk));
                    should_refresh = true;
                }
                DK::CloseView => {
                    break 'outer;
                }
                DK::Command(command) => {
                    match edit.execute_command(command) {
                        Ok(status) => edit.set_status(status),
                        Err(error) => {
                            log::trace!("error: {}", error);
                            return Err(error).context("DK::Command");
                        }
                    }
                    should_refresh = true;
                }
                DK::CommandLine => {
                    edit.enter_command_mode();
                    should_refresh = true;
                }
                DK::Noop => {
                    should_refresh = true;
                }
            }
        }
    }
    Ok(())
}
