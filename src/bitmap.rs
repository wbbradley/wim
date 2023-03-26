use crate::buf::buf_fmt;
use crate::color::Color;
use crate::glyph::{FormattedGlyph, Glyph};
use crate::prelude::*;

#[derive(Debug)]
pub struct Bitmap {
    size: Size,
    cursor: Option<Pos>,
    bmp: Vec<FormattedGlyph>,
}
const DEFAULT_GLYPH: FormattedGlyph = FormattedGlyph {
    glyph: Glyph { ch: ' ' },
    fg: Color::None,
    bg: Color::None,
};

impl Bitmap {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            cursor: None,
            bmp: vec![DEFAULT_GLYPH; size.area()],
        }
    }
    pub fn clear(&mut self) {
        self.bmp.truncate(0);
        for _ in 0..self.size.area() {
            self.bmp.push(DEFAULT_GLYPH);
        }
    }
    pub fn write_to(&self, buf: &mut Buf) {
        let size = self.size;
        for y in 0..size.height {
            buf_fmt!(buf, "\x1b[{};{}H", y + 1, 1);
            for x in 0..size.width {
                let formatted_glyph: &FormattedGlyph = &self.bmp[x + y * size.width];
                formatted_glyph.write_to(buf);
            }
        }
    }
}

pub struct BitmapView<'a> {
    bitmap: &'a mut Bitmap,
    frame: Rect,
}

impl<'a> BitmapView<'a> {
    pub fn get_size(&self) -> Size {
        self.frame.size()
    }
    pub fn set_cursor(&mut self, pos: Pos) {
        // bmp.cursor = buf, "\x1b[{};{}H", pos.y + 1, pos.x + 1);
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
        self.bitmap.bmp[pos.x + pos.y * self.bitmap.size.width] = FormattedGlyph {
            glyph,
            fg: Color::None,
            bg: Color::None,
        };
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
            self.set_glyph(pos, Glyph { ch });
            pos.x += 1;
            count += 1;
        }
        count
    }
}
