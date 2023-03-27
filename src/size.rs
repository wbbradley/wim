use crate::types::{Coord, Pos};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Size {
    pub width: Coord,
    pub height: Coord,
}

impl Size {
    pub fn zero() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }
    pub fn area(self) -> usize {
        self.width * self.height
    }
    pub fn contains(self, pos: Pos) -> bool {
        (0..self.width).contains(&pos.x) && (0..self.height).contains(&pos.y)
    }
}

impl From<Pos> for Size {
    fn from(coord: Pos) -> Self {
        Self {
            width: coord.x,
            height: coord.y,
        }
    }
}
