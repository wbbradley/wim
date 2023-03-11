use crate::mode::Mode;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Command {
    Open { filename: String },
    Save,
    Execute(String),
    Move(Direction),
    SwitchMode(Mode),
    FocusUp,
    FocusDown,
    FocusLeft,
    FocusRight,
    FocusPrevious,
    FocusCommandLine,
    JoinLines,
    NewlineAbove,
    NewlineBelow,
    DeleteForwards,
    DeleteBackwards,
    Many(Vec<Command>),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
