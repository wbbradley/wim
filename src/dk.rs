use crate::prelude::*;
use crate::view::ViewKey;

#[derive(Clone, Debug)]
pub enum DK {
    Key(Key),
    Dispatch(Target, Message),
    Sequence(Vec<DK>),
}

#[derive(Clone, Debug)]
pub enum Message {
    SendKey(Key),
    Command { name: String, args: Vec<Variant> },
}

pub trait ToDK {
    fn vk(self, view_key: ViewKey) -> DK;
}

impl ToDK for Message {
    fn vk(self, view_key: ViewKey) -> DK {
        DK::Dispatch(Target::View(view_key), self)
    }
}

impl<T> From<DK> for Result<DK, T> {
    #[inline]
    fn from(dk: DK) -> Self {
        Self::Ok(dk)
    }
}

impl<T: Into<String>> From<T> for Variant {
    fn from(s: T) -> Self {
        Self::String(s.into())
    }
}

impl From<ViewKey> for Variant {
    fn from(vk: ViewKey) -> Self {
        Self::ViewKey(vk)
    }
}
