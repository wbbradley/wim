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
    EscSeq(u8, u8),
    Up,
    Down,
    Left,
    Right,
    Ctrl(char),
    Ascii(char),
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::Up => write!(f, "<Up>"),
            Key::Down => write!(f, "<Down>"),
            Key::Left => write!(f, "<Left>"),
            Key::Right => write!(f, "<Right>"),
            Key::Ctrl(ch) => write!(f, "<C-{}>", ch),
            Key::Ascii(ch) => write!(f, "{}", ch),
            Key::Esc => write!(f, "<Esc>"),
            Key::EscSeq(a, b) => write!(f, "<Esc-{}-{}>", *a as char, *b as char),
        }
    }
}

pub fn read_key() -> Option<Key> {
    match read_u8() {
        Some(0x1b) => match (read_u8(), read_u8()) {
            (Some(b'['), Some(b'A')) => Some(Key::Up),
            (Some(b'['), Some(b'B')) => Some(Key::Down),
            (Some(b'['), Some(b'C')) => Some(Key::Right),
            (Some(b'['), Some(b'D')) => Some(Key::Left),
            (Some(a), Some(b)) => Some(Key::EscSeq(a, b)),
            (_, _) => Some(Key::Esc),
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
