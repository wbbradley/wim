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
    pub fn area(&self) -> Coord {
        self.width * self.height
    }
}
