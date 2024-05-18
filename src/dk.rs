use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum DK {
    Key(Key),
    Dispatch(Target, Message),
    Sequence(Vec<DK>),
}

impl<T> From<DK> for Result<DK, T> {
    #[inline]
    fn from(dk: DK) -> Self {
        Self::Ok(dk)
    }
}
