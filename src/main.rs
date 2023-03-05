use crate::buf::Buf;
use crate::editor::Editor;
use crate::read::{read_key, Key};
use crate::utils::put;
use log::LevelFilter;
use std::env;
use std::io;

mod buf;
mod editor;
mod files;
mod read;
mod row;
mod termios;
mod types;
mod utils;

pub static VERSION: &str = "v0.1.0";

fn main() -> io::Result<()> {
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
                    Key::Ascii(ch) => edit.insert_char(ch),
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
