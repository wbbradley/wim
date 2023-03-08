use crate::mode::Mode;

#[derive(Debug)]
pub enum Command {
    Open { filename: String },
    Save,
    Move(Direction),
    SwitchMode(Mode),
}

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
