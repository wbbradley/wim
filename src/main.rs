use crate::read::{ctrl_key, read_char, Key};
use crate::termios::Termios;
use crate::utils::put;
mod files;
mod read;
mod termios;
mod utils;

fn editor_refresh_screen() {
    put!("\x1b[2J");
    put!("\x1b[H");
    editor_draw_rows();
    put!("\x1b[H");
}

fn editor_draw_rows() {
    for _ in 0..24 {
        put!("~\r\n");
    }
}

fn main() {
    let _termios = Termios::enter_raw_mode();
    loop {
        editor_refresh_screen();
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
