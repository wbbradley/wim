use crate::prelude::*;
use crate::types::Pos;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Variant {
    Ref(ViewKey, String),
    Int(i64),
    ViewKey(ViewKey),
    String(String),
    Bool(bool),
    Pos(Pos),
}

impl From<bool> for Variant {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<&str> for Variant {
    fn from(s: &str) -> Self {
        Self::String(s.into())
    }
}

impl From<ViewKey> for Variant {
    fn from(vk: ViewKey) -> Self {
        Self::ViewKey(vk)
    }
}
