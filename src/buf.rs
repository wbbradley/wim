use std::fmt::{write, Arguments, Result, Write};

#[derive(Clone, Default)]
pub struct Buf(Vec<u8>);

impl Buf {
    pub fn truncate(&mut self, s: usize) {
        self.0.truncate(s)
    }
    pub fn push_char(&mut self, ch: char) {
        self.0.extend(ch.encode_utf8(&mut [0u8; 4]).as_bytes());
    }
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }
    #[allow(dead_code)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> Extend<&'a u8> for Buf {
    fn extend<I: IntoIterator<Item = &'a u8>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl Buf {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Write for Buf {
    fn write_str(&mut self, s: &str) -> Result {
        self.0.extend(s.as_bytes());
        Ok(())
    }
    fn write_char(&mut self, ch: char) -> Result {
        self.write_str(ch.encode_utf8(&mut [0; 4]))
    }
    fn write_fmt(mut self: &mut Self, args: Arguments<'_>) -> Result {
        write(&mut self, args)
    }
}
