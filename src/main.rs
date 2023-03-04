use crate::read::read_char;
use crate::termios::Termios;
mod files;
mod read;
mod termios;
mod utils;

fn main() {
    let _termios = Termios::enter_raw_mode();
    loop {
        if let Some(ch) = read_char() {
            if unsafe { libc::iscntrl(ch as i32) } != 0 {
                println!("{}\r", ch as i32);
            } else {
                println!("{} ('{}')\r", ch as i32, ch);
            }
            if ch == 'q' {
                break;
            }
        }
    }
}
