use crate::read::{ctrl_key, read_char, Key};
use crate::termios::Termios;
mod files;
mod read;
mod termios;
mod utils;

fn editor_refresh_screen() {
    unsafe {
        libc::write(
            libc::STDOUT_FILENO,
            b"\x1b[2J" as *const u8 as *const libc::c_void,
            4,
        );
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
}
