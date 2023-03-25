use crate::prelude::*;

pub struct Line<'a> {
    bmp: &'a mut BitmapView<'a>,
    pos: Pos,
}

impl<'a> Line<'a> {
    pub fn new(bmp: &'a mut BitmapView<'a>, pos: Pos) -> Self {
        assert!(bmp.get_size().contains(pos));
        let len = bmp.get_size().width;
        Self { bmp, pos }
    }
    fn max_line_length(&self) -> usize {
        self.bmp.get_size().width
    }
    fn cur_dist_to_end(&self) -> usize {
        self.max_line_length() - self.pos.x
    }
    pub fn append_str<T>(&mut self, b: &str) {
        let max_pos = self.bmp.get_size().width;
        for ch in b.chars() {
            if self.pos.x >= max_pos {
                log::trace!("stopped appending to Line prematurely [ch={}]", ch);
                break;
            }
            self.bmp.set_glyph(self.pos, Glyph { ch });
            self.pos.x += 1;
        }
    }
    pub fn remaining_space(&self) -> usize {
        let mll = self.max_line_length();
        if mll > self.pos.x {
            mll - self.pos.x
        } else {
            0
        }
    }
    pub fn end_with_str(&mut self, s: &str) {
        let count = s.chars().count();
        if self.remaining_space() >= count {
            let mll = self.max_line_length();
            self.pos.x = mll - count;
            for ch in s.chars() {
                self.bmp.set_glyph(self.pos, Glyph { ch });
                self.pos.x += 1;
            }
        } else {
            log::trace!("ran out of space to put '{}' at the end of a line.", s);
        }
    }
}

macro_rules! line_fmt {
    ($line:expr, $($args:expr),+) => {{
        let mut stackbuf = [0u8; 4*1024];
        let formatted: &str = stackfmt::fmt_truncate(&mut stackbuf, format_args!($($args),+));
        $line.append_str(formatted);
        formatted.len()
    }};
}
pub(crate) use line_fmt;
