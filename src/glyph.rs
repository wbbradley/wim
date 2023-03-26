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
    pub fn new(glyph: Glyph, fg: Color, bg: Color) -> Self {
        Self { glyph, fg, bg }
    }
    pub const fn from_char(ch: char) -> Self {
        Self {
            glyph: Glyph { ch },
            fg: Color::None,
            bg: Color::None,
        }
    }
    pub fn write_to(&self, buf: &mut Buf) {
        buf_fmt!(buf, "{}", self.glyph.ch);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Formatting {
    val: u32,
}

// const FormattingBold: u32 = 1;
// const FormattingUnderling: u32 = 2;

impl From<char> for Glyph {
    fn from(ch: char) -> Self {
        Self { ch }
    }
}

impl From<char> for FormattedGlyph {
    fn from(ch: char) -> Self {
        Self {
            glyph: ch.into(),
            fg: Color::None,
            bg: Color::None,
        }
    }
}
