use crate::prelude::*;
use crate::view::ViewKey;

#[derive(Clone, Debug)]
pub enum DK {
    Key(Key),
    Dispatch(Target, Message),
    Sequence(Vec<DK>),
}

pub trait ToDK {
    fn vk(self, view_key: ViewKey) -> DK;
}

impl<T> From<DK> for Result<DK, T> {
    #[inline]
    fn from(dk: DK) -> Self {
        Self::Ok(dk)
    }
}
