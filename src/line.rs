use crate::buf::{Buf, ToBufBytes, BLANKS};

#[derive(Debug)]
pub struct Line<'a> {
    buf: &'a mut Buf,
    start_index: usize,
    pub max_line_length: usize,
}

impl<'a> Line<'a> {
    pub fn new(buf: &'a mut Buf, max_line_length: usize) -> Self {
        let len = buf.len();
        Self {
            buf,
            max_line_length,
            start_index: len,
        }
    }
    pub fn cur_offset(&self) -> usize {
        self.buf.len() - self.start_index
    }
    pub fn flush(&mut self) {
        if self.max_line_length >= self.cur_offset() {
            self.buf.append(&BLANKS[..self.remaining_space()]);
        }
    }
    pub fn append<T>(&mut self, b: T)
    where
        T: ToBufBytes,
    {
        self.buf.append(b)
    }
    pub fn remaining_space(&self) -> usize {
        // TODO: have this calculate visible chars, not u8's left.
        self.max_line_length - self.cur_offset()
    }
    pub fn end_with<T>(&mut self, s: T)
    where
        T: ToBufBytes,
    {
        let b = s.to_bytes();
        if self.remaining_space() > b.len() {
            let spaces_needed = self.remaining_space() - b.len();
            self.buf.append(&BLANKS[..spaces_needed]);
            self.buf.append(b);
        } else {
            log::trace!("ran out of space to put {:?} at the end of a line.", b);
        }
    }
}

impl<'a> Drop for Line<'a> {
    fn drop(&mut self) {
        self.flush()
    }
}

macro_rules! line_fmt {
    ($line:expr, $($args:expr),+) => {{
        let mut stackbuf = [0u8; 4*1024];
        let formatted: &str = stackfmt::fmt_truncate(&mut stackbuf, format_args!($($args),+));
        $line.append(formatted);
        formatted.len()
    }};
}
pub(crate) use line_fmt;
