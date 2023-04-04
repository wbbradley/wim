use crate::prelude::*;
use crate::rel::Rel;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TextObj {
    pos: Pos,
    obj_mod: Option<ObjMod>,
    noun: Noun,
    rel: Rel,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjMod {
    Inner,
    A,
    Motion,
}
