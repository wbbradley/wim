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

impl Buf {
    pub fn truncate(&mut self) {
        self.b.truncate(0);
    }
    pub fn append(&mut self, text: &str) {
        self.b.extend_from_slice(text.as_bytes());
    }
    pub fn append_with_max_len(&mut self, text: &str, max_len: usize) {
        let slice = text.as_bytes();
        self.b.extend_from_slice(&slice[0..max_len]);
    }
    pub fn write_to(&self, fd: libc::c_int) {
        unsafe { libc::write(fd, self.b.as_ptr() as *const libc::c_void, self.b.len()) };
    }
}
