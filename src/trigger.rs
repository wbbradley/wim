use crate::command::Command;

#[derive(Debug)]
pub enum Trigger {
    Command(Command),
    Key(Key),
    Exit,
    Noop,
}
