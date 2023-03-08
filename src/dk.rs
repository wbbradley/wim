use crate::command::Command;
use crate::error::Error;
use crate::read::Key;

#[allow(dead_code)]
pub enum DK {
    CommandLine,
    Expansion(Vec<Key>),
    Command(Command),
    Err(Error),
    CloseView,
    Noop,
}
