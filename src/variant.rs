use crate::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Variant {
    Ref(ViewKey, String),
    Int(i64),
    ViewKey(ViewKey),
    String(String),
    Bool(bool),
    Pos(Pos),
    Target(Target),
}

impl From<i64> for Variant {
    fn from(b: i64) -> Self {
        Self::Int(b)
    }
}

impl From<bool> for Variant {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<Target> for Variant {
    fn from(b: Target) -> Self {
        Self::Target(b)
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
