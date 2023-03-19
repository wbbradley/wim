use crate::prelude::*;
use crate::types::Pos;
use crate::view::ViewKey;
use rune::Any;

#[derive(Any, Clone, Debug)]
pub enum DK {
    #[rune(constructor)]
    Key(#[rune(get)] Key),
    #[rune(constructor)]
    Dispatch(#[rune(get)] Option<ViewKey>, #[rune(get)] Message),
    #[rune(constructor)]
    Sequence(#[rune(get)] Vec<DK>),
}

#[derive(Any, Clone, Debug)]
pub enum Message {
    #[rune(constructor)]
    SendKey(#[rune(get)] Key),
    #[rune(constructor)]
    Command {
        #[rune(get)]
        name: String,
        #[rune(get)]
        args: Vec<CallArg>,
    },
}

pub trait ToDK {
    fn vk(self, view_key: ViewKey) -> DK;
}

impl ToDK for Message {
    fn vk(self, view_key: ViewKey) -> DK {
        DK::Dispatch(Some(view_key), self)
    }
}

impl<T> From<DK> for Result<DK, T> {
    #[inline]
    fn from(dk: DK) -> Self {
        Self::Ok(dk)
    }
}

#[derive(Any, Clone, Debug)]
pub enum CallArg {
    #[rune(constructor)]
    Ref(#[rune(get)] ViewKey, #[rune(get)] String),
    #[rune(constructor)]
    Int(#[rune(get)] i64),
    #[rune(constructor)]
    ViewKey(#[rune(get)] ViewKey),
    #[rune(constructor)]
    Float(#[rune(get)] f64),
    #[rune(constructor)]
    String(#[rune(get)] String),
    #[rune(constructor)]
    Bool(#[rune(get)] bool),
    #[rune(constructor)]
    Pos(#[rune(get)] Pos),
}

impl<T: Into<String>> From<T> for CallArg {
    fn from(s: T) -> Self {
        Self::String(s.into())
    }
}

impl From<ViewKey> for CallArg {
    fn from(vk: ViewKey) -> Self {
        Self::ViewKey(vk)
    }
}
