use crate::command::Command;
use crate::key::Key;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Trigger {
    Command(Command),
    Key(Key),
    Exit,
    Noop,
}
