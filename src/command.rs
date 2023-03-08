use crate::mode::Mode;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Command {
    Open { filename: String },
    Save,
    Move(Direction),
    SwitchMode(Mode),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
