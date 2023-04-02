use crate::error::Result;
use std::fmt::{write, Arguments, Result as FmtResult, Write};

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
    pub fn write_to_file(&self, filename: &str) -> Result<()> {
        use crate::error::ErrorContext;
        use std::fs::File;
        use std::io::prelude::*;
        let mut file = File::create(filename)?;
        file.write_all(self.as_bytes())
            .context("write_all in write_to_file")
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
    fn write_str(&mut self, s: &str) -> FmtResult {
        self.0.extend(s.as_bytes());
        Ok(())
    }
    fn write_char(&mut self, ch: char) -> FmtResult {
        self.write_str(ch.encode_utf8(&mut [0; 4]))
    }
    fn write_fmt(mut self: &mut Self, args: Arguments<'_>) -> FmtResult {
        write(&mut self, args)
    }
}
