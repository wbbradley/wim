use crate::buf::Buf;
use crate::command::Command;
use crate::dk::DK;
use crate::editor::Editor;
use crate::error::Error;
use crate::key::Key;
use crate::plugin::Plugin;
use crate::read::read_key;
use crate::termios::Termios;
use crate::types::Rect;
use crate::view::View;
use anyhow::Context as AnyhowContext;
use log::LevelFilter;
use rune::termcolor::{ColorChoice, StandardStream};
use rune::{Context, Diagnostics, FromValue, Source, Sources, Vm};
use std::collections::VecDeque;
use std::env;
use std::sync::Arc;
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
mod widechar_width;

pub static VERSION: &str = "v0.1.0";

fn load_plugin() -> anyhow::Result<Plugin> {
    let rune_context = Context::with_default_modules()?;
    let rune_runtime = Arc::new(rune_context.runtime());
    let mut sources = Sources::new();
    sources.insert(Source::new("plugin", std::fs::read_to_string("wimrc.fn")?));

    let mut diagnostics = Diagnostics::new();

    let result = rune::prepare(&mut sources)
        .with_context(&rune_context)
        .with_diagnostics(&mut diagnostics)
        .build();

    if !diagnostics.is_empty() {
        let mut writer = StandardStream::stderr(ColorChoice::Always);
        diagnostics.emit(&mut writer, &sources)?;
    }

    let unit = result?;
    let mut vm = Vm::new(rune_runtime, Arc::new(unit));
    let output = vm.call(["add"], (10i64, 20i64))?;
    let output = i64::from_value(output)?;

    println!("{}", output);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    //Result<()> {
    let termios = Termios::enter_raw_mode();
    match std::panic::catch_unwind(move || run_app(termios)) {
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

fn run_app(termios: Termios) -> anyhow::Result<()> {
    simple_logging::log_to_file("wim.log", LevelFilter::Trace)?;
    let args: Vec<String> = env::args().collect();
    log::trace!("wim run with args: {:?}", args);

    let plugin = load_plugin()?;
    let frame: Rect = termios.get_window_size().into();

    let mut edit = Editor::new(termios);
    if args.len() > 1 {
        edit.execute_command(Command::Open {
            filename: args[1].clone(),
        })?;
    }
    let mut buf = Buf::default();
    let mut should_refresh = true;
    let mut keys: VecDeque<Key> = Default::default();
    let mut dks: VecDeque<DK> = Default::default();

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
                    dks.push_front(edit.handle_key(key)?);
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
