use crate::color::{BgColor, FgColor};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Format {
    pub fg: FgColor,
    pub bg: BgColor,
}

impl Format {
    pub const fn none() -> Self {
        Self {
            fg: FgColor::None,
            bg: BgColor::None,
        }
    }
    pub const fn selected() -> Self {
        Self {
            fg: FgColor::Black,
            bg: BgColor::White,
        }
    }
}
