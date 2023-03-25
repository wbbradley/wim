use crate::color::Color;
use crate::glyph::{FormattedGlyph, Glyph};
use crate::prelude::*;

#[derive(Debug)]
pub struct Bitmap {
    size: Size,
    cursor: Option<Pos>,
    bmp: Vec<FormattedGlyph>,
}

impl Bitmap {
    fn new(size: Size) -> Self {
        Self {
            size,
            cursor: None,
            bmp: vec![
                FormattedGlyph {
                    glyph: Glyph { ch: ' ' },
                    fg: Color::None,
                    bg: Color::None,
                };
                size.area()
            ],
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
}
