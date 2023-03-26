use crate::buf::{buf_fmt, Buf};
use crate::color::Color;
use crate::glyph::{FormattedGlyph, Glyph};
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Bitmap {
    size: Size,
    cursor: Option<Pos>,
    glyphs: Vec<FormattedGlyph>,
}
const DEFAULT_GLYPH: FormattedGlyph = FormattedGlyph::from_char(' ');

impl Bitmap {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            cursor: None,
            glyphs: vec![DEFAULT_GLYPH; size.area()],
        }
    }
    pub fn clear(&mut self) {
        self.glyphs.truncate(0);
        self.cursor = None;
        for _ in 0..self.size.area() {
            self.glyphs.push(DEFAULT_GLYPH);
        }
    }
    #[allow(dead_code)]
    pub fn write_to(&self, buf: &mut Buf) {
        let size = self.size;
        for y in 0..size.height {
            buf_fmt!(buf, "\x1b[{};{}H", y + 1, 1);
            for x in 0..size.width {
                let formatted_glyph: &FormattedGlyph = &self.glyphs[x + y * size.width];
                formatted_glyph.encode_utf8_to_buf(buf);
            }
        }
    }
    pub fn get_cursor(&self) -> Option<Pos> {
        self.cursor
    }
    pub fn diff(bmp_last: &Self, bmp: &Self, buf: &mut Buf) {
        assert!(bmp_last.size == bmp.size);
        let size = bmp.size;
        for y in 0..size.height {
            let line_range = y * size.width..(y + 1) * size.width;
            let line_changed = bmp_last.glyphs[line_range.clone()] != bmp.glyphs[line_range];

            if line_changed {
                buf_fmt!(buf, "\x1b[{};{}H", y + 1, 1);
                for x in 0..size.width {
                    let formatted_glyph: &FormattedGlyph = &bmp.glyphs[x + y * size.width];
                    formatted_glyph.encode_utf8_to_buf(buf);
                }
            }
        }
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
    pub fn set_glyph(&mut self, pos: Pos, glyph: Glyph) {
        if !self.get_size().contains(pos) {
            log::warn!(
                "attempt to set_glyph beyond bitmap view boundary [pos={:?}, glyph={:?}, frame={:?}, size={:?}]",
                pos,
                glyph,
                self.frame,
                self.bitmap.size
            );
            return;
        }
        let pos = pos + self.frame.top_left();
        self.bitmap.glyphs[pos.x + pos.y * self.bitmap.size.width] =
            FormattedGlyph::new(glyph, Color::None, Color::None);
    }
    pub fn append_chars_at_pos<T>(&mut self, pos: &mut Pos, chs: T)
    where
        T: Iterator<Item = char>,
    {
        pos.x += self.append_chars_at(*pos, chs);
    }
    pub fn append_chars_at<T>(&mut self, mut pos: Pos, chs: T) -> usize
    where
        T: Iterator<Item = char>,
    {
        let max_pos = self.get_size().width;
        let mut count = 0;
        for ch in chs {
            if pos.x >= max_pos {
                break;
            }
            self.set_glyph(pos, ch.into());
            pos.x += 1;
            count += 1;
        }
        count
    }
    pub fn end_line_with_str(&mut self, mut pos: Pos, s: &str) {
        let count = s.chars().count();
        if self.frame.width <= pos.x {
            return;
        }
        let remaining_space = self.frame.width - pos.x;
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
    ($bmp:expr, $pos:expr, $($args:expr),+) => {{
        let mut stackbuf = [0u8; 4*1024];
        let formatted: &str = stackfmt::fmt_truncate(&mut stackbuf, format_args!($($args),+));
        $bmp.append_chars_at_pos(&mut $pos, formatted.chars());
    }};
}
pub(crate) use bmp_fmt_at;
