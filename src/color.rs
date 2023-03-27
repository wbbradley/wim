use std::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Color {
    None,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
    Rgb { r: u8, g: u8, b: u8 },
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::None => write!(f, "\x1b[39m"),
            Color::Rgb { r, g, b } => write!(f, "\x1b[38;2;{};{};{}m", r, g, b),
            Color::Black => write!(f, "\x1b[30m"),
            Color::Red => write!(f, "\x1b[31m"),
            Color::Green => write!(f, "\x1b[32m"),
            Color::Yellow => write!(f, "\x1b[33m"),
            Color::Blue => write!(f, "\x1b[34m"),
            Color::Purple => write!(f, "\x1b[35m"),
            Color::Cyan => write!(f, "\x1b[36m"),
            Color::White => write!(f, "\x1b[37m"),
        }
    }
}

/*
pub struct FgColorToAnsiIter {
    stackbuf: [u8; 32],
}

impl IntoIter<Item = u8, IntoIter = FgColorToAnsiIter> for FgColorChange {
    pub fn into_iter(self) -> Self {
        self
    }
}

impl Iterator for FgColorToAnsiIter {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
    }
}
*/
