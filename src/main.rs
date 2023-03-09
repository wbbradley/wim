use crate::buf::Buf;
use crate::command::Command;
use crate::dk::DK;
use crate::editor::Editor;
use crate::error::Result;
use crate::read::{read_key, Key};
use crate::termios::Termios;
use crate::types::Rect;
use crate::utils::put;
use crate::view::View;
use log::LevelFilter;
use std::collections::VecDeque;
use std::env;
mod buf;
mod command;
mod commandline;
mod dk;
mod doc;
mod docview;
mod editor;
mod error;
mod files;
mod keygen;
mod mode;
mod noun;
mod read;
mod row;
mod status;
mod termios;
mod trigger;
mod types;
mod utils;
mod view;
mod widechar_width;

pub static VERSION: &str = "v0.1.0";

fn main() -> Result<()> {
    simple_logging::log_to_file("wim.log", LevelFilter::Trace)?;
    let args: Vec<String> = env::args().collect();
    log::trace!("wim run with args: {:?}", args);

    let termios = Termios::enter_raw_mode();
    let frame: Rect = termios.get_window_size().into();

    let mut edit = Editor::new(termios);
    if args.len() > 1 {
        edit.dispatch_command(Command::Open {
            filename: args[1].clone(),
        })?;
    }
    let mut buf = Buf::default();
    let mut should_refresh = true;
    let mut keys: VecDeque<Key> = Default::default();
    loop {
        if should_refresh {
            edit.layout(frame);
            buf.truncate();
            edit.display(&mut buf);
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
        match edit.dispatch_key(key) {
            DK::Expansion(next_keys) => {
                next_keys.iter().rev().for_each(|&key| keys.push_front(key));
                should_refresh = true;
            }
            DK::Err(error) => return Err(error),
            DK::CloseView => {
                break;
            }
            DK::Command(command) => {
                edit.dispatch_command(command)?;
                should_refresh = true;
            }
            DK::CommandLine => {
                edit.enter_command_mode();
                should_refresh = true;
            }
            DK::Noop => {}
        };
    }
    put!("\x1b[2J");
    put!("\x1b[H");
    Ok(())
}
