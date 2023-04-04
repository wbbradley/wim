use crate::pos::Pos;
use crate::read::read_u8;
use crate::size::Size;
use crate::types::{Coord, SafeCoordCast};
use crate::utils::{die, put};
use std::sync::{Arc, Mutex};

pub struct Termios {
    pub orig: libc::termios,
    in_raw_mode: Arc<Mutex<bool>>,
}

impl Termios {
    fn new() -> Self {
        let fd = libc::STDIN_FILENO;
        let mut termios = Self {
            orig: unsafe { std::mem::zeroed() },
            in_raw_mode: Arc::new(Mutex::new(false)),
        };
        let ret = unsafe { libc::tcgetattr(fd, &mut termios.orig as *mut libc::termios) };
        if ret == -1 {
            die!("[Termios::new] unable to tcgetattr");
        }
        termios
    }
    fn enable_raw_mode(&mut self) -> anyhow::Result<()> {
        let mut raw = libc::termios {
            c_cc: self.orig.c_cc,
            c_cflag: self.orig.c_cflag,
            c_iflag: self.orig.c_iflag,
            c_ispeed: self.orig.c_ispeed,
            c_ospeed: self.orig.c_ospeed,
            c_lflag: self.orig.c_lflag,
            c_oflag: self.orig.c_oflag,
        };
        raw.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
        raw.c_oflag &= !(libc::OPOST);
        raw.c_cflag |= libc::CS8;
        raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
        raw.c_cc[libc::VMIN] = 0;
        raw.c_cc[libc::VTIME] = 1;
        if unsafe {
            libc::tcsetattr(
                libc::STDIN_FILENO,
                libc::TCSAFLUSH,
                &mut raw as *mut libc::termios,
            )
        } == -1
        {
            die!("tcsetattr failed");
        }
        let mut in_raw_mode = self.in_raw_mode.lock().unwrap();
        *in_raw_mode = true;
        Ok(())
    }
    pub fn enter_raw_mode() -> Self {
        let mut termios = Self::new();
        termios.enable_raw_mode().unwrap();
        // Clear the screen to begin.
        put!(libc::STDOUT_FILENO, "\x1b[2J");
        termios
    }
    pub fn exit_raw_mode(&self) {
        let mut in_raw_mode = self.in_raw_mode.lock().unwrap();
        if *in_raw_mode {
            put!(libc::STDOUT_FILENO, "\x1b[2J\x1b[H\x1b[0m");
            if unsafe {
                libc::tcsetattr(
                    libc::STDIN_FILENO,
                    libc::TCSAFLUSH,
                    &self.orig as *const libc::termios,
                )
            } == -1
            {
                die!("Termios::drop");
            }
            put!(libc::STDOUT_FILENO, "\x1b[2J\x1b[H\x1b[0m");
            *in_raw_mode = false;
        }
    }

    pub fn get_window_size() -> Size {
        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        if unsafe {
            libc::ioctl(
                libc::STDIN_FILENO,
                libc::TIOCGWINSZ,
                &mut ws as *mut libc::winsize as *mut libc::c_void,
            )
        } == -1
            || ws.ws_col == 0
        {
            if put!(libc::STDOUT_FILENO, "\x1b[999C\x1b[999B") != 12 {
                read_u8();
                Size {
                    width: 80,
                    height: 24,
                }
            } else if let Some(coord) = get_cursor_position() {
                coord.into()
            } else {
                Size {
                    width: 80,
                    height: 24,
                }
            }
        } else {
            Size {
                width: ws.ws_col.as_coord(),
                height: ws.ws_row.as_coord(),
            }
        }
    }
}

impl Drop for Termios {
    fn drop(&mut self) {
        self.exit_raw_mode();
    }
}

fn get_cursor_position() -> Option<Pos> {
    let mut buf = [0u8; 32];
    let mut i: usize = 0;

    // Write the "get position" command.
    if put!(libc::STDOUT_FILENO, "\x1b[6n") != 4 {
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
    let y: Coord = lexical::parse(&buf[0..semicolon_position]).unwrap();
    let x: Coord = lexical::parse(&buf[semicolon_position + 1..]).unwrap();
    Some(Pos { y, x })
}
