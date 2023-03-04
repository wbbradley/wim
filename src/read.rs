use crate::utils::{die, Errno};

pub trait Key {
    fn to_keycode(self) -> i32;
}

impl Key for char {
    fn to_keycode(self) -> i32 {
        self as i32
    }
}
#[inline]
pub fn ctrl_key<T>(k: T) -> i32
where
    T: Key,
{
    k.to_keycode() & 0x1f
}

pub fn read_char() -> Option<char> {
    let mut ch: char = '\0';
    let ret = unsafe {
        libc::read(
            libc::STDIN_FILENO,
            &mut ch as *mut char as *mut libc::c_void,
            1,
        )
    };
    if ret == 1 {
        Some(ch)
    } else if ret == -1 && !Errno::latest().is_eagain() {
        die!("failed to read_char!");
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
