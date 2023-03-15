use crate::buf::Buf;
use crate::command::Command;
use crate::dk::DK;
use crate::editor::Editor;
use crate::error::Error;
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
    let plugin = load_plugin()?;
    let termios = Termios::enter_raw_mode();
    // UnwindSafe
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe/*lying*/(move || {
        run_app(termios, plugin)
    })) {
        Ok(result) => result.context("during run_app"),
        Err(panic) => match panic.downcast::<String>() {
            Ok(error) => {
                log::error!("panic: {:?}", error);
                Err(Error::new("panic!!")).context("panic")
            }
            Err(_) => {
                log::error!("panic with unknown type.");
                Err(Error::new("panic!!")).context("panic")
            }
        },
    }
}

fn run_app(termios: Termios, plugin: PluginRef) -> anyhow::Result<()> {
    simple_logging::log_to_file("wim.log", LevelFilter::Trace)?;
    let args: Vec<String> = env::args().collect();
    log::trace!("wim run with args: {:?}", args);

    let frame: Rect = termios.get_window_size().into();

    let mut edit = Editor::new(termios, plugin);
    if args.len() > 1 {
        edit.execute_command(Command::Open {
            filename: args[1].clone(),
        })?;
    }
    let mut buf = Buf::default();
    let mut should_refresh = true;
    let mut keys: VecDeque<Key> = Default::default();
    let mut dks: VecDeque<DK> = Default::default();
    let mut cur_keys: Vec<Key> = Default::default();

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
        while let Some(dk) = dks.pop_front() {
            log::trace!("handling dk: {:?}", dk);
            match dk {
                DK::Key(key) => {
                    cur_keys.push(key);
                    std::mem::drop(dk);
                    let next_dk = edit.handle_keys(&cur_keys)?;
                    if DK::AmbiguousKeys == next_dk {
                        // Swallow ambiguous key sequences.
                        log::trace!("seeing ambiguous keys with {:?}", cur_keys);
                        continue;
                    }
                    cur_keys.truncate(0);
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
                DK::AmbiguousKeys => {
                    panic!("This should be swallowed above.");
                }
            }
        }
    }
    Ok(())
}
