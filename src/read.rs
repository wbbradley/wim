use crate::utils::{die, Errno};

#[inline]
pub fn is_ctrl_key(k: u8) -> bool {
    k & 0x1f == k
}
#[inline]
pub fn decode_ctrl_key(k: u8) -> char {
    assert!(is_ctrl_key(k));
    (k + b'a' - 1) as char
}

#[derive(Copy, Clone, Debug)]
pub enum Key {
    Esc,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Ctrl(char),
    Ascii(char),
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::ArrowUp => write!(f, "<Up>"),
            Key::ArrowDown => write!(f, "<Down>"),
            Key::ArrowLeft => write!(f, "<Left>"),
            Key::ArrowRight => write!(f, "<Right>"),
            Key::Ctrl(ch) => write!(f, "<C-{}>", ch),
            Key::Ascii(ch) => write!(f, "{}", ch),
            Key::Esc => write!(f, "<Esc>"),
        }
    }
}

pub fn read_key() -> Option<Key> {
    match read_u8() {
        Some(0x1b) => match (read_u8(), read_u8()) {
            (Some(b'['), Some(b'A')) => Some(Key::ArrowUp),
            (Some(b'['), Some(b'B')) => Some(Key::ArrowDown),
            (Some(b'['), Some(b'C')) => Some(Key::ArrowRight),
            (Some(b'['), Some(b'D')) => Some(Key::ArrowLeft),
            (_, _) => Some(Key::Esc), //todo!(),
        },
        Some(ch) if is_ctrl_key(ch) => Some(Key::Ctrl(decode_ctrl_key(ch))),
        Some(ch) => {
            if ch < 128 {
                Some(Key::Ascii(ch as char))
            } else {
                panic!("unhandled key '{}'", ch as char);
            }
        }
        None => None,
    }
}

pub fn read_u8() -> Option<u8> {
    let mut ch: u8 = 0;
    let ret = unsafe {
        libc::read(
            libc::STDIN_FILENO,
            &mut ch as *mut u8 as *mut libc::c_void,
            1,
        )
    };
    if ret == 1 {
        Some(ch)
    } else if ret == -1 && !Errno::latest().is_eagain() {
        die!("failed to read_u8!");
    } else {
        None
    }
}
