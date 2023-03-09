use crate::mode::Mode;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Command {
    Open { filename: String },
    Save,
    Move(Direction),
    SwitchMode(Mode),
    ChangeFocus(FocusTarget),
    JoinLines,
    NewlineAbove,
    NewlineBelow,
    DeleteForwards,
    DeleteBackwards,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum FocusTarget {
    CommandLine,
    Up,
    Down,
    Left,
    Right,
    Previous,
    Next,
}
