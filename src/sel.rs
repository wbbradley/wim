use crate::types::Pos;

#[derive(Debug, Clone, Copy)]
pub struct Sel {
    pub start: Pos,
    pub end: Pos,
}

impl Sel {
    pub fn from_pos(pos: Pos) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }
}
