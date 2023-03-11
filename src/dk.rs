use crate::command::Command;
use crate::key::Key;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum DK {
    Key(Key),
    CommandLine,
    Command(Command),
    Sequence(Vec<DK>),
    CloseView,
    Noop,
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
