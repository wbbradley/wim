use crate::buf::Buf;
use crate::editor::{Editor, Status};
use crate::noun::Noun;
use crate::read::{read_key, Key};
use crate::utils::put;
use anyhow::Result;
use log::LevelFilter;
use std::env;
use std::time::{Duration, Instant};
mod buf;
mod doc;
mod editor;
mod error;
mod files;
mod noun;
mod read;
mod row;
mod termios;
mod types;
mod utils;

pub static VERSION: &str = "v0.1.0";

fn main() -> Result<()> {
    simple_logging::log_to_file("wim.log", LevelFilter::Trace)?;
    let args: Vec<String> = env::args().collect();
    log::trace!("wim run with args: {:?}", args);

    let mut edit = Editor::new();
    if args.len() > 1 {
        edit.open(args[1].clone())?;
    }
    let mut buf = Buf::default();
    let mut updated = true;
    loop {
        if updated {
            edit.scroll();
            edit.refresh_screen(&mut buf);
        }
        match read_key() {
            Some(ch) => {
                edit.set_last_key(ch);
                match ch {
                    Key::Esc => log::trace!("you pressed Esc!?"),
                    Key::EscSeq1(_) => continue,
                    Key::EscSeq2(_, _) => continue,
                    Key::Ctrl('w') => break,
                    Key::Ctrl('s') => {
                        let status = edit.save_file();
                        edit.set_status(match status {
                            Ok(status) => status,
                            Err(error) => Status::Message {
                                message: format!("error during save: {}", error),
                                expiry: Instant::now() + Duration::from_secs(4),
                            },
                        });
                    }
                    Key::Del => continue,
                    Key::Left => edit.move_cursor(-1, 0),
                    Key::Down => edit.move_cursor(0, 1),
                    Key::Up => edit.move_cursor(0, -1),
                    Key::Right => edit.move_cursor(1, 0),
                    Key::PageDown => edit.move_cursor(0, edit.screen_size.height),
                    Key::PageUp => edit.move_cursor(0, -edit.screen_size.height),
                    Key::Home => edit.jump_cursor(Some(0), None),
                    Key::End => edit.jump_cursor(Some(i64::MAX), None),
                    Key::Ascii('h') => edit.move_cursor(-1, 0),
                    Key::Ascii('j') => edit.move_cursor(0, 1),
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
                updated = true;
            }
            None => {
                updated = edit.expired_status();
            }
        }
    }
    put!("\x1b[2J");
    put!("\x1b[H");
    Ok(())
}
