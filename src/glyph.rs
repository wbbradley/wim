use crate::buf::Buf;
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
    pub fn encode_utf8_to_buf(&self, buf: &mut Buf) {
        let mut b = [0; 4];
        self.glyph.ch.encode_utf8(&mut b);
        buf.extend(b);
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
