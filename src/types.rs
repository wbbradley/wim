#[cfg(test)]
#[macro_use]
extern crate quickcheck;

pub type Coord = usize;
pub type RelCoord = isize;

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

#[derive(Copy, Clone, Debug)]
pub struct Size {
    pub width: Coord,
    pub height: Coord,
}

impl Size {
    pub fn area(self) -> usize {
        self.width * self.height
    }
    pub fn contains(self, pos: Pos) -> bool {
        (0..self.width).contains(&pos.x) && (0..self.height).contains(&pos.y)
    }
}

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub struct Pos {
    pub x: Coord,
    pub y: Coord,
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

impl From<&Pos> for Pos {
    fn from(v: &Pos) -> Self {
        *v
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
mod tests {
    quickcheck! {
        fn prop(r: Rect, p: Pos) -> bool {
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
}
