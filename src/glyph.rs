use crate::buf::Buf;
use crate::color::Color;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Glyph {
    ch: char,
    format: Format,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Format {
    fg: Color,
    bg: Color,
}

impl Format {
    pub fn none() -> Self {
        Self {
            fg: Color::None,
            bg: Color::None,
        }
    }
}

impl Glyph {
    pub fn new(ch: char, format: Format) -> Self {
        Self { ch, format }
    }
    pub const fn from_char(ch: char) -> Self {
        Self {
            ch,
            format: Format::none(),
        }
    }
    pub fn encode_utf8_to_buf(
        &self,
        buf: &mut Buf,
        Format {
            fg: last_fg,
            bg: last_bg,
        }: Format,
    ) -> Format {
        if last_fg != self.format.fg {
            write!(buf, "{}", self.format.fg);
        }
        buf.push_char(self.ch);
        self.format
    }
}

impl From<char> for Glyph {
    fn from(ch: char) -> Self {
        Self {
            ch,
            format: Format::none(),
        }
    }
}
