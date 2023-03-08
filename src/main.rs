use crate::buf::Buf;

use crate::command::{Command, Direction};
use crate::editor::Editor;
use crate::noun::Noun;
use crate::read::{read_key, Key};
use crate::status::Status;
use crate::types::{Coord, RelCoord};
use crate::utils::put;
use anyhow::Result;
use log::LevelFilter;
use std::env;
use std::time::{Duration, Instant};
mod buf;
mod command;
mod doc;
mod editor;
mod error;
mod files;
mod keygen;
mod noun;
mod read;
mod row;
mod status;
mod termios;
mod types;
mod utils;

pub static VERSION: &str = "v0.1.0";

pub enum Trigger {
    Command(Command),
    Exit,
    Noop,
}

fn translate_key(key: Option<Key>) -> Result<Trigger> {
    match key {
        Key::Esc => Trigger::Noop,
        Key::EscSeq1(_) => Trigger::Noop,
        Key::EscSeq2(_, _) => Trigger::Noop,
        Key::Ctrl('w') => Trigger::Exit,
        Key::Ctrl('s') => Trigger::Command(Command::Save),
        Key::Del => Trigger::Noop,
        Key::Left => Trigger::Command(Command::Move(Direction::Left)), // (edit.move_cursor(-1, 0),
        Key::Down => edit.move_cursor(0, 1),
        Key::Up => edit.move_cursor(0, -1),
        Key::Right => edit.move_cursor(1, 0),
        Key::PageDown => edit.move_cursor(0, edit.screen_size.height as RelCoord),
        Key::PageUp => edit.move_cursor(0, -(edit.screen_size.height as RelCoord)),
        Key::Home => edit.jump_cursor(Some(0), None),
        Key::Ascii(':') => edit.enter_command_mode(),
        Key::End => edit.jump_cursor(Some(Coord::MAX), None),
        Key::Ascii('h') => edit.move_cursor(-1, 0),
        Key::Ascii('j') => edit.move_cursor(0, 1),
        Key::Ascii('J') => edit.join_line(),
        Key::Ascii('k') => edit.move_cursor(0, -1),
        Key::Ascii('l') => edit.move_cursor(1, 0),
        Key::Ascii('o') => edit.insert_newline_below(),
        Key::Ascii('O') => edit.insert_newline_above(),
        Key::Ascii('x') => edit.delete_forwards(Noun::Char),
        Key::Ascii('X') => edit.delete_backwards(Noun::Char),
        Key::Ascii(ch) => edit.insert_char(ch),
        Key::Ctrl('u') => edit.delete_backwards(Noun::Line),
        Key::Ctrl('k') => edit.delete_forwards(Noun::Line),
        Key::Ctrl(_) => (),
        Key::Function(_) => (),
        Key::PrintScreen => (),
        Key::Backspace => (),
    };
}

fn main() -> Result<()> {
    simple_logging::log_to_file("wim.log", LevelFilter::Trace)?;
    let args: Vec<String> = env::args().collect();
    log::trace!("wim run with args: {:?}", args);

    let mut edit = Editor::new();
    if args.len() > 1 {
        edit.dispatch_command(Command::Open {
            filename: args[1].clone(),
        })?;
    }
    let mut buf = Buf::default();
    let mut should_refresh = true;
    let mut triggers: Vec<Trigger> = Vec::new();
    loop {
        if should_refresh {
            edit.refresh_screen(&mut buf);
            should_refresh = false;
        }
        let trigger = if triggers.len() != 0 {
            triggers.remove(0)
        } else {
            translate_key(edit.read_key())?
        };
        match trigger {
            Trigger::Exit => {
                break;
            }
            Trigger::Noop => {
                continue;
            }
            Trigger::Command(command) => {
                edit.dispatch_command(command)?;
                should_refresh = true;
            }
        };
    }
    put!("\x1b[2J");
    put!("\x1b[H");
    Ok(())
}
