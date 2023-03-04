use crate::types::SafeCoordCast;

pub struct Buf {
    b: Vec<u8>,
}

impl Default for Buf {
    fn default() -> Self {
        let mut b = Vec::new();
        b.reserve(2 << 16);
        Self { b }
    }
}

pub trait ToBufBytes {
    fn to_bytes(&self) -> &[u8];
}

impl ToBufBytes for &[u8] {
    fn to_bytes(&self) -> &[u8] {
        self
    }
}

impl ToBufBytes for &str {
    fn to_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ToBufBytes for &String {
    fn to_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Buf {
    pub fn truncate(&mut self) {
        self.b.truncate(0);
    }
    pub fn append<T>(&mut self, text: T)
    where
        T: ToBufBytes,
    {
        self.b.extend_from_slice(text.to_bytes());
    }
    pub fn append_with_max_len<T, U>(&mut self, text: T, max_len: U)
    where
        T: ToBufBytes,
        U: SafeCoordCast,
    {
        let slice = text.to_bytes();
        self.b
            .extend_from_slice(&slice[0..std::cmp::min(max_len.as_coord() as usize, slice.len())]);
    }
    pub fn write_to(&self, fd: libc::c_int) {
        unsafe { libc::write(fd, self.b.as_ptr() as *const libc::c_void, self.b.len()) };
    }
}

impl ToBufBytes for Buf {
    fn to_bytes(&self) -> &[u8] {
        &self.b
    }
}
