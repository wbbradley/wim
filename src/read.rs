use crate::key::Key;
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

pub fn read_key(reader: &mut impl Iterator<Item = u8>) -> Option<Key> {
    // trace!("reading a key...");
    if let Some(ch) = reader.next() {
        if ch == 0x1b {
            match (reader.next(), reader.next()) {
                (Some(b'['), Some(b'A')) => Some(Key::Up),
                (Some(b'['), Some(b'B')) => Some(Key::Down),
                (Some(b'['), Some(b'C')) => Some(Key::Right),
                (Some(b'['), Some(b'D')) => Some(Key::Left),
                (Some(b'['), Some(b'H')) => Some(Key::Home),
                (Some(b'['), Some(b'F')) => Some(Key::End),
                (Some(b'['), Some(a)) if a.is_ascii_digit() => match reader.next() {
                    Some(b) if b.is_ascii_digit() => match (a, b, reader.next()) {
                        (b'1', b'5', Some(b'~')) => Some(Key::Function(5)),
                        (b'1', b'7', Some(b'~')) => Some(Key::Function(6)),
                        (b'1', b'8', Some(b'~')) => Some(Key::Function(7)),
                        (b'1', b'9', Some(b'~')) => Some(Key::Function(8)),
                        (b'2', b'0', Some(b'~')) => Some(Key::Function(9)),
                        (b'2', b'1', Some(b'~')) => Some(Key::Function(10)),
                        (b'2', b'3', Some(b'~')) => Some(Key::Function(11)),
                        (b'2', b'4', Some(b'~')) => Some(Key::Function(12)),
                        _ => Some(Key::Esc),
                    },
                    Some(b';') => match (a, b';', reader.next(), reader.next()) {
                        (b'1', b';', Some(b'2'), Some(b'P')) => Some(Key::PrintScreen),
                        _ => Some(Key::Esc),
                    },
                    Some(b'~') => match a {
                        b'1' => Some(Key::Home),
                        b'3' => Some(Key::Del),
                        b'4' => Some(Key::End),
                        b'5' => Some(Key::PageUp),
                        b'6' => Some(Key::PageDown),
                        b'7' => Some(Key::Home),
                        b'8' => Some(Key::End),
                        _ => Some(Key::Esc),
                    },
                    _ => Some(Key::Esc),
                },
                (Some(b'O'), Some(b'H')) => Some(Key::Home),
                (Some(b'O'), Some(b'F')) => Some(Key::End),
                (Some(b'O'), Some(b'P')) => Some(Key::Function(1)),
                (Some(b'O'), Some(b'Q')) => Some(Key::Function(2)),
                (Some(b'O'), Some(b'R')) => Some(Key::Function(3)),
                (Some(b'O'), Some(b'S')) => Some(Key::Function(4)),
                (Some(a), Some(b)) => Some(Key::EscSeq2(a, b)),
                (Some(a), None) => Some(Key::EscSeq1(a)),
                (_, _) => Some(Key::Esc),
            }
        } else if is_ctrl_key(ch) {
            if ch == 13 {
                Some(Key::Enter)
            } else {
                let d = decode_ctrl_key(ch);
                if d == 'c' {
                    panic!("C-c pressed. Quitting...");
                }
                Some(Key::Ctrl(d))
            }
        } else if ch < 127 {
            Some(Key::Utf8(ch as char))
        } else if ch == 127 {
            Some(Key::Backspace)
        } else if ch & 0b11100000 == 0b11000000 {
            log::trace!("saw {:#04x}", ch);
            let ch = ch as u32;
            let b = reader.next()? as u32;
            log::trace!("saw {:#04x}", b);
            assert!(b & 0b11000000 == 0b10000000);
            Some(Key::Utf8(char::from_u32(
                ((ch & 0b00011111) << 6) | (b & 0b00111111),
            )?))
        } else if ch & 0b11110000 == 0b11100000 {
            log::trace!("saw {:#04x}", ch);
            let ch = ch as u32;
            let b = reader.next()? as u32;
            log::trace!("saw {:#04x}", b);
            assert!(b & 0b11000000 == 0b10000000);
            let c = reader.next()? as u32;
            log::trace!("saw {:#04x}", c);
            assert!(c & 0b11000000 == 0b10000000);
            let val = ((ch & 0b00001111) << 12) | ((b & 0b00111111) << 6) | (c & 0b00111111);
            log::trace!("saw {:#08x} s", val);
            Some(Key::Utf8(char::from_u32(val)?))
        } else if ch & 0b11111000 == 0b11110000 {
            let ch = ch as u32;
            log::trace!("saw {:#04x}", ch);
            let b = reader.next()? as u32;
            log::trace!("saw {:#04x}", b);
            assert!(b & 0b11000000 == 0b10000000);
            let c = reader.next()? as u32;
            log::trace!("saw {:#04x}", c);
            assert!(c & 0b11000000 == 0b10000000);
            let d = reader.next()? as u32;
            log::trace!("saw {:#04x}", d);
            assert!(d & 0b11000000 == 0b10000000);
            Some(Key::Utf8(char::from_u32(
                ((ch & 0b00001111) << 18)
                    | ((b & 0b00111111) << 12)
                    | ((c & 0b00111111) << 6)
                    | (d & 0b00111111),
            )?))
        } else {
            panic!("unhandled key '{}'", ch as char);
        }
    } else {
        None
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
