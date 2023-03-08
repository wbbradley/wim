use crate::buf::Buf;
use crate::command::Command;
use crate::editor::Editor;
use crate::error::Result;
use crate::read::{read_key, Key};
use crate::termios::Termios;
use crate::types::Rect;
use crate::utils::put;
use crate::view::{View, DK};
use log::LevelFilter;
use std::collections::VecDeque;
use std::env;
mod buf;
mod command;
mod doc;
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

pub static VERSION: &str = "v0.1.0";

/*
fn translate_key(key: Option<Key>) -> Result<Vec<Trigger>> {
    if let Some(key) = key {
        let mut new_keys: Vec<Key> = Vec::new();
        match key {
            Key::Esc => triggers.push(Trigger::Noop),
            Key::EscSeq1(_) => Ok(Trigger::Noop),
            Key::EscSeq2(_, _) => Ok(Trigger::Noop),
            Key::Ctrl('w') => Ok(Trigger::Exit),
            Key::Ctrl('s') => Ok(Trigger::Command(Command::Save)),
            Key::Del => Ok(Trigger::Noop),
            Key::Left => Ok(Trigger::Command(Command::Move(Direction::Left))), // (edit.move_cursor(-1, 0),
            Key::Down => Ok(Trigger::Command(Command::Move(Direction::Down))), // (edit.move_cursor(-1, 0),
            Key::Up => Ok(Trigger::Command(Command::Move(Direction::Up))), // (edit.move_cursor(-1, 0),
            Key::Right => Ok(Trigger::Command(Command::Move(Direction::Right))), // (edit.move_cursor(-1, 0),
            Key::PageDown => {
                triggers.extend_from_slice(&[push(Command::Ok(Trigger::Command(Command::Moveedit.move_cursor(0, edit.screen_size.height as RelCoord),
            /*
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
            Key::Backspace => (),*/
            _ => { return Err(Error::not_impl(format!("translate_key {:?}", key))); },
        };
        if triggers.len() == 0 {
            Ok(vec![Trigger::Noop])
        } else {
            Ok(triggers)
        }
    } else {
        Ok(Trigger::Noop)
    }
}
*/
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
            DK::Mapping(next_keys) => {
                next_keys.iter().rev().for_each(|&key| keys.push_front(key));
                should_refresh = true;
            }
            DK::Err(error) => return Err(error),
            DK::CloseView => {
                break;
            }
        };
    }
    put!("\x1b[2J");
    put!("\x1b[H");
    Ok(())
}
