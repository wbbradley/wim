use crate::command::Command;
use crate::key::Key;

#[allow(dead_code)]
pub enum DK {
    CommandLine,
    Expansion(Vec<Key>),
    Command(Command),
    CloseView,
    Noop,
}
