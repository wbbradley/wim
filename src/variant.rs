use crate::prelude::*;
use crate::types::Pos;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Variant {
    Ref(ViewKey, String),
    Int(i64),
    ViewKey(ViewKey),
    Float(f64),
    String(String),
    Bool(bool),
    Pos(Pos),
}
