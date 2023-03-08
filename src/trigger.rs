use crate::command::Command;
use crate::read::Key;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Trigger {
    Command(Command),
    Key(Key),
    Exit,
    Noop,
}
