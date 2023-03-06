use crate::error::Result;
use libc::strerror;
use std::ffi::CStr;
use std::fs::File;
use std::io::BufRead;
use std::io::{BufReader, Lines};
use std::path::Path;

#[allow(dead_code)]
#[cfg(any(target_os = "linux"))]
pub mod errors {
    use libc::__errno_location;
    pub fn get_errno() -> c_int {
        unsafe { *__errno_location() }
    }
}

#[allow(dead_code)]
#[cfg(any(target_os = "macos"))]
pub mod errors {
    use libc::__error;
    pub fn get_errno() -> libc::c_int {
        unsafe { *__error() }
    }
}

macro_rules! die {
    ($message:expr) => {
        panic!("error: {}: {}", $message, $crate::utils::Errno::latest())
    };
    ($fmt:expr, $($args:expr),+) => {{
        let user_message = format!($fmt, $($args),+);
        panic!("error: {}: {}", $crate::utils::Errno::latest(), user_message)
    }};
}
pub(crate) use die;

macro_rules! put {
    ($($args:expr),+) => {{
        let mut buf = [0u8; 1024];
        let formatted: &str = stackfmt::fmt_truncate(&mut buf, format_args!($($args),+));
        unsafe {
            libc::write(
                libc::STDOUT_FILENO,
                formatted.as_ptr() as *const libc::c_void,
                formatted.len(),
            )
        }
    }};
}
pub(crate) use put;

#[derive(Copy, Clone)]
pub struct Errno {
    errno: libc::c_int,
}

impl Errno {
    pub fn latest() -> Self {
        Self {
            errno: errors::get_errno(),
        }
    }
    pub fn is_enoent(self) -> bool {
        self.errno == libc::ENOENT
    }
    pub fn is_eagain(self) -> bool {
        self.errno == libc::EAGAIN
    }
}

impl std::fmt::Display for Errno {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = (*self).into();
        write!(f, "{}", s)
    }
}

impl From<Errno> for String {
    fn from(errno: Errno) -> Self {
        String::from_utf8_lossy(unsafe { CStr::from_ptr(strerror(errno.errno)) }.to_bytes())
            .to_string()
    }
}

pub fn read_lines<P>(filename: P) -> Result<Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}
