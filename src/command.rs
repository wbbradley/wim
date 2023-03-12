use crate::mode::Mode;
use crate::noun::Noun;
use crate::rel::Rel;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Command {
    Open { filename: String },
    Save,
    Execute(String),
    Move(Direction),
    MoveRel(Noun, Rel),
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

#[derive(Clone, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
