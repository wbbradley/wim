use crate::utils::{die, Errno};

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
