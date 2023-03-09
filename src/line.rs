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
            self.buf
                .append(&BLANKS[..self.max_line_length - self.cur_offset()]);
        }
    }
    pub fn append<T>(&mut self, b: T)
    where
        T: ToBufBytes,
    {
        self.buf.append(b)
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
