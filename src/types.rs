use crate::pos::Pos;
use crate::size::Size;
use std::ops::RangeBounds;

pub type Coord = usize;
pub type RelCoord = isize;

#[cfg(test)]
use quickcheck::Arbitrary;

#[allow(clippy::wrong_self_convention)]
pub trait SafeCoordCast {
    fn as_coord(self) -> Coord;
}

impl SafeCoordCast for u64 {
    #[inline]
    fn as_coord(self) -> Coord {
        self as Coord
    }
}

impl SafeCoordCast for i64 {
    #[inline]
    fn as_coord(self) -> Coord {
        assert!(self >= 0);
        self as Coord
    }
}

impl SafeCoordCast for i32 {
    #[inline]
    fn as_coord(self) -> Coord {
        assert!(self >= 0);
        self as Coord
    }
}

impl SafeCoordCast for usize {
    #[inline]
    fn as_coord(self) -> Coord {
        self as Coord
    }
}

impl SafeCoordCast for i16 {
    #[inline]
    fn as_coord(self) -> Coord {
        assert!(self >= 0);
        self as Coord
    }
}

impl SafeCoordCast for u16 {
    #[inline]
    fn as_coord(self) -> Coord {
        self as Coord
    }
}

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub struct Rect {
    pub x: Coord,
    pub y: Coord,
    pub width: Coord,
    pub height: Coord,
}

#[allow(dead_code)]
impl Rect {
    pub fn contains(self, pos: Pos) -> bool {
        (self.x..self.x + self.width).contains(&pos.x)
            && (self.y..self.y + self.height).contains(&pos.y)
    }
    pub fn zero() -> Self {
        unsafe { std::mem::zeroed() }
    }
    pub fn area(self) -> Coord {
        self.width * self.height
    }
    pub fn top_left(self) -> Pos {
        Pos {
            x: self.x,
            y: self.y,
        }
    }
    pub fn mid_x(self) -> Coord {
        ((self.x * 2) + self.width) / 2
    }
    pub fn mid_y(self) -> Coord {
        ((self.y * 2) + self.height) / 2
    }
    pub fn max_x(self) -> Coord {
        self.x + self.width
    }
    pub fn max_y(self) -> Coord {
        self.y + self.height
    }
    pub fn size(self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

impl std::ops::Add<Pos> for Rect {
    type Output = Rect;
    fn add(self, pos: Pos) -> Self {
        Self {
            x: self.x + pos.x,
            y: self.y + pos.y,
            width: self.width,
            height: self.height,
        }
    }
}

impl From<Size> for Rect {
    fn from(s: Size) -> Self {
        Self {
            x: 0,
            y: 0,
            width: s.width,
            height: s.height,
        }
    }
}

#[cfg(test)]
impl Arbitrary for Rect {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            x: Coord::arbitrary(g) % 10000,
            y: Coord::arbitrary(g) % 10000,
            width: Coord::arbitrary(g) % 10000,
            height: Coord::arbitrary(g) % 10000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[quickcheck]
    fn pos_compare(p: Pos, q: Pos) -> bool {
        if p > q {
            q < p
        } else {
            p <= q
        }
    }

    #[quickcheck]
    fn rect_pos_addition(r: Rect, p: Pos) -> bool {
        r + p == {
            let q = r.top_left() + p;
            Rect {
                x: q.x,
                y: q.y,
                width: r.width,
                height: r.height,
            }
        }
    }
}
