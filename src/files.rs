use crate::utils::Errno;
use libc::c_int;
use std::ffi::CString;
use std::fmt;

pub struct FileDescriptor {
    fd: c_int,
}

#[allow(dead_code)]
impl FileDescriptor {
    pub fn open(filename: &str, flags: c_int, mode: c_int) -> Result<Self, Errno> {
        println!("opening {}...", filename);
        match CString::new(filename) {
            Ok(cstr_filename) => {
                let ret = unsafe { libc::open(cstr_filename.as_ptr(), flags, mode) };
                if ret != -1 {
                    Ok(Self { fd: ret })
                } else {
                    Err(Errno::latest())
                }
            }
            Err(error) => {
                panic!(
                    "error with filename: [filename={:?}, error={:?}]",
                    filename, error
                );
            }
        }
    }
    /// # Safety
    /// Knowing is half the battle.
    pub unsafe fn write(&self, buf: *const libc::c_void, count: libc::size_t) -> libc::ssize_t {
        libc::write(self.fd, buf, count)
    }
    pub fn lseek(&self, offset: libc::off_t, whence: libc::c_int) -> libc::off_t {
        unsafe { libc::lseek(self.fd, offset, whence) }
    }
    /// # Safety
    /// Be cool, stay in school.
    pub unsafe fn read(&self, buf: *mut libc::c_void, count: libc::size_t) -> libc::ssize_t {
        libc::read(self.fd, buf, count)
    }
}

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

impl fmt::Display for FileDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileDescriptor(fd={})", self.fd)
    }
}

#[allow(dead_code)]
fn rename(old: String, new: String) -> c_int {
    match CString::new(old) {
        Ok(old) => match CString::new(new) {
            Ok(new) => unsafe { libc::rename(old.as_ptr(), new.as_ptr()) },
            Err(error) => {
                panic!("error: {:?}", error);
            }
        },
        Err(error) => {
            panic!("error: {:?}", error);
        }
    }
}
