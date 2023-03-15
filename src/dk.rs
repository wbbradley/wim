use crate::command::Command;
use crate::key::Key;
use rune::Any;

#[allow(dead_code)]
#[derive(Any, Clone, Debug, Eq, PartialEq)]
pub enum DK {
    #[rune(constructor)]
    Key(#[rune(get)] Key),
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
    #[rune(constructor)]
    AmbiguousKeys,
}

impl From<Command> for DK {
    #[inline]
    fn from(command: Command) -> Self {
        Self::Command(command)
    }
}

impl<T> From<Command> for Result<DK, T> {
    #[inline]
    fn from(command: Command) -> Self {
        Self::Ok(DK::Command(command))
    }
}

impl<T> From<DK> for Result<DK, T> {
    #[inline]
    fn from(dk: DK) -> Self {
        Self::Ok(dk)
    }
}
