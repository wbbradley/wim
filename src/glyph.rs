use crate::buf::{buf_fmt, Buf};
use crate::color::Color;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Glyph {
    ch: char,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FormattedGlyph {
    glyph: Glyph,
    fg: Color,
    bg: Color,
}

impl FormattedGlyph {
    pub fn write_to(&self, buf: &mut Buf) {
        buf_fmt!(buf, "{}", self.glyph.ch);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Formatting {
    val: u32,
}

const FormattingBold: u32 = 1;
const FormattingUnderling: u32 = 2;
