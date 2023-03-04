use crate::read::{ctrl_key, read_char, read_u8, Key};
use crate::termios::Termios;
use crate::utils::put;
use log::LevelFilter;
use std::io;
mod files;
mod read;
mod termios;
mod utils;

static VERSION: &str = "v0.1.0";

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct Size {
    cols: usize,
    rows: usize,
}

#[allow(dead_code)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Coord {
    col: usize,
    row: usize,
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
    let row: usize = lexical::parse(&buf[0..semicolon_position]).unwrap();
    let col: usize = lexical::parse(&buf[semicolon_position + 1..]).unwrap();
    Some(Coord { row, col })
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
            cols: ws.ws_col as usize,
            rows: ws.ws_row as usize,
        }
    }
}

#[allow(dead_code)]
struct Editor {
    termios: Termios,
    screen_size: Size,
    cursor: Coord,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            termios: Termios::enter_raw_mode(),
            screen_size: get_window_size(),
            cursor: Coord::default(),
        }
    }
    fn refresh_screen(&self, buf: &mut ABuf) {
        buf.truncate();
        buf.append("\x1b[?25l\x1b[H");
        self.draw_rows(buf);
        buf_fmt!(buf, "\x1b[{};{}H", self.cursor.row, self.cursor.col);
        buf.append("\x1b[H\x1b[?25h");
        buf.write_to(libc::STDIN_FILENO);
    }

    fn draw_rows(&self, buf: &mut ABuf) {
        for y in 0..self.screen_size.rows {
            if y == self.screen_size.rows / 3 {
                let welcome = format!("Wim editor -- version {}", VERSION);
                let mut welcome_len = welcome.len();
                if welcome_len > self.screen_size.cols {
                    welcome_len = self.screen_size.cols;
                }
                let mut padding = (self.screen_size.cols - welcome_len) / 2;
                if padding != 0 {
                    buf.append("~");
                    padding -= 1;
                }
                for _ in 0..padding {
                    buf.append(" ");
                }
                buf.append_with_max_len(&welcome, welcome_len);
            } else {
                buf.append("~");
            }

            buf.append("\x1b[K");
            if y < self.screen_size.rows - 1 {
                buf.append("\r\n");
            }
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        println!("Closing wim.\r\n  Screen size was {:?}\r", self.screen_size);
    }
}

pub struct ABuf {
    b: Vec<u8>,
}

impl Default for ABuf {
    fn default() -> Self {
        let mut b = Vec::new();
        b.reserve(2 << 16);
        Self { b }
    }
}

macro_rules! buf_fmt {
    ($buf:expr, $($args:expr),+) => {{
        let mut buf = [0u8; 1024];
        let formatted: &str = stackfmt::fmt_truncate(&mut buf, format_args!($($args),+));
        $buf.append(&formatted);
    }};
}
pub(crate) use buf_fmt;

impl ABuf {
    pub fn truncate(&mut self) {
        self.b.truncate(0);
    }
    pub fn append_bytes(&mut self, text: &[u8]) {
        self.b.extend_from_slice(text);
    }
    pub fn append(&mut self, text: &str) {
        self.b.extend_from_slice(text.as_bytes());
    }
    pub fn append_with_max_len(&mut self, text: &str, max_len: usize) {
        let slice = text.as_bytes();
        self.b.extend_from_slice(&slice[0..max_len]);
    }
    pub fn write_to(&self, fd: libc::c_int) {
        unsafe { libc::write(fd, self.b.as_ptr() as *const libc::c_void, self.b.len()) };
    }
}

fn main() -> io::Result<()> {
    simple_logging::log_to_file("wim.log", LevelFilter::Trace)?;

    let edit = Editor::new();

    let mut buf = ABuf::default();
    loop {
        edit.refresh_screen(&mut buf);
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
