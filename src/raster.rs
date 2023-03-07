use crate::buf::{Buf, ToBufBytes};
use std::default::Default;

#[derive(Default)]
pub struct Raster {
    buf: Buf,
    breaks: Vec<usize>,
}

impl Raster {
    pub fn truncate(&mut self) {
        self.buf.truncate();
        self.breaks.truncate();
    }
    pub fn append<T>(&mut self, text: T)
    where
        T: ToBufBytes,
    {
        self.buf.append(text);
    }
    pub fn add_break(&mut self) {
        self.breaks.push(self.buf.len());
    }
}
