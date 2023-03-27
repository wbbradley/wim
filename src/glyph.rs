use crate::buf::Buf;
use crate::error::{ErrorContext, Result};
use crate::format::Format;
use std::fmt::Write;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Glyph {
    pub ch: char,
    pub format: Format,
}

impl Glyph {
    pub fn from_char(ch: char) -> Self {
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
    ) -> Result<Format> {
        if last_fg != self.format.fg {
            write!(buf, "{}", self.format.fg).context("write fg")?
        }
        if last_bg != self.format.bg {
            write!(buf, "{}", self.format.bg).context("write bg")?
        }
        buf.push_char(self.ch);
        Ok(self.format)
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
