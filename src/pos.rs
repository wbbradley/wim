use crate::types::{Coord, Rect};
use std::cmp::Ordering;
use std::ops::{Bound, RangeBounds};

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub struct Pos {
    pub x: Coord,
    pub y: Coord,
}

pub struct PosRange {
    start: Pos,
    end: Pos,
    end_inclusive: bool,
    whole_lines: bool,
}

impl Pos {
    pub fn range(start: Self, end: Self) -> PosRange {
        assert!(start <= end);
        PosRange {
            start,
            end,
            end_inclusive: false,
            whole_lines: false,
        }
    }
    pub fn range_inclusive(start: Self, end: Self) -> PosRange {
        assert!(start <= end);
        PosRange {
            start,
            end,
            end_inclusive: true,
            whole_lines: false,
        }
    }
    pub fn range_lines(start: Coord, end: Coord) -> PosRange {
        assert!(start < end);
        PosRange {
            start: Pos { x: 0, y: start },
            end: Pos { x: 0, y: end },
            end_inclusive: false,
            whole_lines: true,
        }
    }
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
