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
            bmp: {
                let mut v = Vec::new();
                v.reserve(size.area());
                v
            },
        }
    }
    fn set_glyph(&mut self, pos: Pos, glyph: Glyph) {
        if !(0..self.size.width).contains(&pos.x) || !(0..self.size.height).contains(&pos.y) {
            log::warn!(
                "attempt to set_glyph beyond bitmap boundary [pos={:?}, glyph={:?}, size={:?}]",
                pos,
                glyph,
                self.size
            );
            return;
        }
        self.bmp[pos.x + pos.y * self.size.width] = FormattedGlyph {
            glyph,
            fg: Color::None,
            bg: Color::None,
        };
    }
}
