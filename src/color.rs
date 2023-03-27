use crate::format::Format;
use std::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FgColor {
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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BgColor {
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

impl fmt::Display for FgColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "\x1b[39m"),
            Self::Rgb { r, g, b } => write!(f, "\x1b[38;2;{};{};{}m", r, g, b),
            Self::Black => write!(f, "\x1b[30m"),
            Self::Red => write!(f, "\x1b[31m"),
            Self::Green => write!(f, "\x1b[32m"),
            Self::Yellow => write!(f, "\x1b[33m"),
            Self::Blue => write!(f, "\x1b[34m"),
            Self::Purple => write!(f, "\x1b[35m"),
            Self::Cyan => write!(f, "\x1b[36m"),
            Self::White => write!(f, "\x1b[37m"),
        }
    }
}

impl fmt::Display for BgColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "\x1b[49m"),
            Self::Rgb { r, g, b } => write!(f, "\x1b[48;2;{};{};{}m", r, g, b),
            Self::Black => write!(f, "\x1b[40m"),
            Self::Red => write!(f, "\x1b[41m"),
            Self::Green => write!(f, "\x1b[42m"),
            Self::Yellow => write!(f, "\x1b[43m"),
            Self::Blue => write!(f, "\x1b[44m"),
            Self::Purple => write!(f, "\x1b[45m"),
            Self::Cyan => write!(f, "\x1b[46m"),
            Self::White => write!(f, "\x1b[47m"),
        }
    }
}

impl From<BgColor> for Format {
    fn from(bg: BgColor) -> Self {
        Self {
            bg,
            fg: FgColor::None,
        }
    }
}

impl From<FgColor> for Format {
    fn from(fg: FgColor) -> Self {
        Self {
            fg,
            bg: BgColor::None,
        }
    }
}

impl std::ops::Add<FgColor> for BgColor {
    type Output = Format;
    #[inline]
    fn add(self, fg: FgColor) -> Format {
        Format { fg, bg: self }
    }
}

impl std::ops::Add<BgColor> for FgColor {
    type Output = Format;
    #[inline]
    fn add(self, bg: BgColor) -> Format {
        Format { fg: self, bg }
    }
}
