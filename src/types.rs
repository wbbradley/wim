use rune::Any;

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

#[derive(Any, Default, Copy, Clone, Debug)]
pub struct Pos {
    pub x: Coord,
    pub y: Coord,
}

#[allow(dead_code)]
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

#[derive(Default, Copy, Clone, Debug)]
pub struct Rect {
    pub x: Coord,
    pub y: Coord,
    pub width: Coord,
    pub height: Coord,
}

#[allow(dead_code)]
impl Rect {
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
