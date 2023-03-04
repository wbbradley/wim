use crate::read::{ctrl_key, read_char, read_u8, Key};
use crate::termios::Termios;
use crate::utils::put;
use log::LevelFilter;
use std::io;
mod files;
mod read;
mod termios;
mod utils;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct Size {
    cols: u16,
    rows: u16,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct Coord {
    col: u16,
    row: u16,
}

impl From<Coord> for Size {
    fn from(coord: Coord) -> Self {
        Self {
            cols: coord.col,
            rows: coord.row,
        }
    }
}

fn get_cursor_position() -> Option<Coord> {
    let mut buf = [0u8; 32];
    let mut i: usize = 0;

    // Write the "get position" command.
    if put!("\x1b[6n") != 4 {
        return None;
    }
    loop {
        if i >= 32 - 1 {
            break;
        }
        if let Some(ch) = read_u8() {
            buf[i] = ch;
            if ch == b'R' {
                break;
            }
            i += 1;
        } else {
            return None;
        }
    }
    buf[i] = 0;
    if buf[0] != 0x1b || buf[1] != b'[' {
        return None;
    }
    let buf = &buf[2..i];
    let semicolon_position = buf.iter().position(|x| *x == b';').unwrap();
    let row: libc::c_int = lexical::parse(&buf[0..semicolon_position]).unwrap();
    let col: libc::c_int = lexical::parse(&buf[semicolon_position + 1..]).unwrap();
    Some(Coord {
        row: row as u16,
        col: col as u16,
    })
}

fn get_window_size() -> Size {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    if unsafe {
        libc::ioctl(
            libc::STDOUT_FILENO,
            libc::TIOCGWINSZ,
            &mut ws as *mut libc::winsize as *mut libc::c_void,
        )
    } == -1
        || ws.ws_col == 0
    {
        if put!("\x1b[999C\x1b[999B") != 12 {
            read_char();
            Size { cols: 80, rows: 24 }
        } else if let Some(coord) = get_cursor_position() {
            coord.into()
        } else {
            Size { cols: 80, rows: 24 }
        }
    } else {
        Size {
            cols: ws.ws_col,
            rows: ws.ws_row,
        }
    }
}

#[allow(dead_code)]
struct Editor {
    termios: Termios,
    screen_size: Size,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            termios: Termios::enter_raw_mode(),
            screen_size: get_window_size(),
        }
    }
    fn refresh_screen(&self) {
        put!("\x1b[2J");
        put!("\x1b[H");
        self.draw_rows();
        put!("\x1b[H");
    }

    fn draw_rows(&self) {
        for _ in 0..self.screen_size.rows - 1 {
            put!("~\r\n");
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        println!("Closing wim.\r\n  Screen size was {:?}\r", self.screen_size);
    }
}

fn main() -> io::Result<()> {
    simple_logging::log_to_file("wim.log", LevelFilter::Trace)?;

    let edit = Editor::new();

    loop {
        edit.refresh_screen();
        if let Some(ch) = read_char() {
            let ch = ch.to_keycode();
            if ch == ctrl_key('q') {
                break;
            }
        }
    }
    put!("\x1b[2J");
    put!("\x1b[H");
    Ok(())
}
