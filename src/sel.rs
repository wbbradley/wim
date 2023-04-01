use crate::types::Pos;

#[allow(dead_code)]
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
