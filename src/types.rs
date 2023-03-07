pub type Coord = usize;
pub type RelCoord = isize;

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

#[derive(Default, Copy, Clone, Debug)]
pub struct Pos {
    pub x: Coord,
    pub y: Coord,
}

impl Pos {
    pub fn clamp(&self, r: &Rect) -> Self {
        Self {
            x: self.x.clamp(r.x, r.x + r.width),
            y: self.y.clamp(r.y, r.y + r.height),
        }
    }
}

impl From<&Pos> for Pos {
    fn from(v: &Pos) -> Self {
        v.clone()
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

#[allow(dead_code)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Rect {
    pub x: Coord,
    pub y: Coord,
    pub width: Coord,
    pub height: Coord,
}

impl Rect {
    pub fn zero() -> Self {
        std::mem::zeroed()
    }
    pub fn area(self) -> Coord {
        self.width * self.height
    }
    fn mid_x(self) -> Coord {
        ((self.x * 2) + self.width) / 2
    }
    fn mid_y(self) -> Coord {
        ((self.y * 2) + self.height) / 2
    }
    fn max_x(self) -> Coord {
        self.x + self.width
    }
    fn max_y(self) -> Coord {
        self.y + self.height
    }
}
