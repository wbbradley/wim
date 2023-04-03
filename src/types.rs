use crate::size::Size;
use std::cmp::Ordering;
use std::ops::{Bound, RangeBounds};

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
pub struct Pos {
    pub x: Coord,
    pub y: Coord,
}

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.y < other.y {
            Ordering::Less
        } else if self.y > other.y {
            Ordering::Greater
        } else if self.x < other.x {
            Ordering::Less
        } else if self.x > other.x {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

#[allow(dead_code)]
impl Pos {
    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
    pub fn clamp(&self, r: &Rect) -> Self {
        Self {
            x: self.x.clamp(r.x, r.x + r.width),
            y: self.y.clamp(r.y, r.y + r.height),
        }
    }
    pub fn get_start_pos(range: &impl RangeBounds<Self>) -> Self {
        match range.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => Pos {
                x: start.x + 1,
                y: start.y,
            },
            Bound::Unbounded => Pos::zero(),
        }
    }
    pub fn get_end_pos(range: &impl RangeBounds<Self>) -> Self {
        match range.end_bound() {
            Bound::Included(&end) => Pos {
                x: end.x + 1,
                y: end.y,
            },
            Bound::Excluded(&end) => end,
            Bound::Unbounded => panic!("unbounded end pos is not implemented..."),
        }
    }
    pub fn inc_x(self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
        }
    }
    pub fn dec_x(self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y,
        }
    }
}

impl std::ops::Add<Pos> for Pos {
    type Output = Self;
    fn add(self, rhs: Pos) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
#[cfg(test)]
impl Arbitrary for Pos {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            x: Coord::arbitrary(g) % 10000,
            y: Coord::arbitrary(g) % 10000,
        }
    }
}

impl From<&Pos> for Pos {
    fn from(v: &Pos) -> Self {
        *v
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
