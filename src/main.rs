use crate::read::{ctrl_key, read_char, Key};
use crate::termios::Termios;
mod files;
mod read;
mod termios;
mod utils;

fn main() {
    let _termios = Termios::enter_raw_mode();
    loop {
        if let Some(ch) = read_char() {
            let ch = ch.to_keycode();
            if unsafe { libc::iscntrl(ch) } != 0 {
                println!("{}\r", ch);
            } else {
                println!("{} ('{}')\r", ch, ch as u8 as char);
            }
            if ch == ctrl_key('q') {
                break;
            }
        }
    }
}
