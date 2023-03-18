use crate::prelude::*;

#[derive(Any, Clone, Debug)]
pub enum DK {
    DK(#[rune(get)] ViewKey, #[rune(get)] Form),
    #[rune(constructor)]
    None,
}

#[derive(Any, Clone, Debug)]
pub enum Form {
    #[rune(constructor)]
    Key(#[rune(get)] Key),
    #[rune(constructor)]
    SendKey(#[rune(get)] Key),
    #[rune(constructor)]
    CommandLine,
    #[rune(constructor)]
    Command(#[rune(get)] Command),
    #[rune(constructor)]
    Sequence(#[rune(get)] Vec<DK>),
    #[rune(constructor)]
    CloseView,
    #[rune(constructor)]
    Noop,
}

pub trait ToDK {
    fn with_view_key(self, view_key: ViewKey) -> DK;
}

impl ToDK for Form {
    fn with_view_key(self, view_key: ViewKey) -> DK {
        DK::DK(view_key, self)
    }
}

impl<T> From<DK> for Result<DK, T> {
    #[inline]
    fn from(dk: DK) -> Self {
        Self::Ok(dk)
    }
}
