use crate::read::{ctrl_key, read_char, Key};
use crate::termios::Termios;
use crate::utils::{die, put};
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

fn get_window_size() -> Size {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(
            libc::STDOUT_FILENO,
            libc::TIOCGWINSZ,
            &mut ws as *mut libc::winsize as *mut libc::c_void,
        ) == -1
            || ws.ws_col == 0
        {
            die!("ioctl failed in get_window_size");
        } else {
            Size {
                cols: ws.ws_col,
                rows: ws.ws_row,
            }
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

fn main() {
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
}
