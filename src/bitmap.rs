use crate::buf::Buf;
use crate::color::{BgColor, FgColor};
use crate::error::{ErrorContext, Result};
use crate::format::Format;
use crate::glyph::Glyph;
use crate::prelude::*;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct Bitmap {
    size: Size,
    cursor: Option<Pos>,
    glyphs: Vec<Glyph>,
    default_glyph: Glyph,
    pub dirty: bool,
}

impl Bitmap {
    pub fn new(size: Size, default_glyph: Glyph) -> Self {
        Self {
            size,
            cursor: None,
            default_glyph,
            glyphs: vec![default_glyph; size.area()],
            dirty: true,
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.size = size;
        self.clear();
    }

    pub fn clear(&mut self) {
        self.cursor = None;
        self.glyphs = vec![self.default_glyph; self.size.area()];
        self.dirty = true;
    }

    pub fn get_cursor(&self) -> Option<Pos> {
        self.cursor
    }

    pub fn diff(bmp_last: &Self, bmp: &Self, buf: &mut Buf) -> Result<()> {
        assert!(bmp_last.size == bmp.size);
        write!(buf, "\x1b[0m").context("clearing graphic rendition")?;
        let mut last_format: Format = Format::none();
        let size = bmp.size;
        for y in 0..size.height {
            let line_range = y * size.width..(y + 1) * size.width;
            let line_changed =
                bmp_last.dirty || bmp_last.glyphs[line_range.clone()] != bmp.glyphs[line_range];

            if line_changed {
                write!(buf, "\x1b[{};{}H", y + 1, 1).context("writing raster start")?;
                for x in 0..size.width {
                    let glyph: &Glyph = &bmp.glyphs[x + y * size.width];
                    last_format = glyph.encode_utf8_to_buf(buf, last_format)?;
                }
            }
        }
        Ok(())
    }
    #[allow(dead_code)]
    pub fn get_glyph(&self, pos: Pos) -> &Glyph {
        &self.glyphs[pos.x + pos.y * self.size.width]
    }
}

pub struct BitmapView<'a> {
    bitmap: &'a mut Bitmap,
    frame: Rect,
}

impl<'a> BitmapView<'a> {
    pub fn new(glyphs: &'a mut Bitmap, frame: Rect) -> Self {
        Self {
            bitmap: glyphs,
            frame,
        }
    }
    pub fn get_size(&self) -> Size {
        self.frame.size()
    }
    pub fn set_cursor(&mut self, pos: Pos) {
        if self.frame.contains(pos) {
            self.bitmap.cursor = Some(self.frame.top_left() + pos);
        }
    }

    fn get_glyph(&mut self, pos: Pos) -> &mut Glyph {
        if !self.get_size().contains(pos) {
            panic!(
                "attempt to get_glyph beyond bitmap view boundary [pos={:?}, frame={:?}]",
                pos, self.frame,
            );
        }
        let pos = pos + self.frame.top_left();
        // trace!("getting glyph at {:?}", pos);
        &mut self.bitmap.glyphs[pos.x + pos.y * self.bitmap.size.width]
    }

    pub fn set_glyph(&mut self, pos: Pos, glyph: Glyph) {
        let target_glyph = self.get_glyph(pos);
        target_glyph.ch = glyph.ch;

        match glyph.format {
            Format {
                fg: FgColor::None,
                bg: BgColor::None,
            } => {}
            Format {
                fg,
                bg: BgColor::None,
            } => {
                target_glyph.format.fg = fg;
            }
            Format {
                fg: FgColor::None,
                bg,
            } => {
                target_glyph.format.bg = bg;
            }
            Format { fg, bg } => {
                target_glyph.format.fg = fg;
                target_glyph.format.bg = bg;
            }
        }
    }

    pub fn set_bg(&mut self, pos: Pos, bg: BgColor) {
        let target_glyph = self.get_glyph(pos);
        target_glyph.format.bg = bg;
    }

    pub fn append_chars_at_pos<T>(&mut self, pos: &mut Pos, chs: T, format: Format)
    where
        T: Iterator<Item = char>,
    {
        pos.x += self.append_chars_at(*pos, chs, format);
    }
    pub fn append_chars_at<T>(&mut self, mut pos: Pos, chs: T, format: Format) -> usize
    where
        T: Iterator<Item = char>,
    {
        let max_pos = self.get_size().width;
        let mut count = 0;
        for ch in chs {
            if pos.x >= max_pos {
                break;
            }
            self.set_glyph(pos, Glyph { ch, format });
            pos.x += 1;
            count += 1;
        }
        count
    }
    pub fn end_line_with_str(&mut self, mut pos: Pos, s: &str) {
        log::trace!("calling end_line_with_str(pos={:?}, s={:?})", pos, s);
        let count = s.chars().count();
        if self.frame.width <= pos.x {
            log::trace!("pos.x is too high!");
            return;
        }
        let remaining_space = self.frame.width - pos.x;
        log::trace!("remaining_space = {}, count = {}", remaining_space, count);
        if remaining_space >= count {
            let mll = self.frame.width;
            pos.x = mll - count;
            for ch in s.chars() {
                self.set_glyph(pos, ch.into());
                pos.x += 1;
            }
        } else {
            log::trace!("ran out of space to put '{}' at the end of a line.", s);
        }
    }
}

macro_rules! bmp_fmt_at {
    ($bmp:expr, $pos:expr, $format:expr, $($args:expr),+) => {{
        let mut stackbuf = [0u8; 4*1024];
        let formatted: &str = stackfmt::fmt_truncate(&mut stackbuf, format_args!($($args),+));
        $bmp.append_chars_at_pos(&mut $pos, formatted.chars(), $format);
    }};
}
pub(crate) use bmp_fmt_at;
