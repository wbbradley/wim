use crate::mode::Mode;
use crate::noun::Noun;
use crate::rel::Rel;
use rune::Any;

#[derive(Any, Clone, Debug, Eq, PartialEq)]
pub enum Command {
    #[rune(constructor)]
    Open {
        #[rune(get)]
        filename: String,
    },
    #[rune(constructor)]
    Save,
    #[rune(constructor)]
    Execute(#[rune(get)] String),
    #[rune(constructor)]
    Move(#[rune(get)] Direction),
    #[rune(constructor)]
    MoveRel(#[rune(get)] Noun, #[rune(get)] Rel),
    #[rune(constructor)]
    SwitchMode(#[rune(get)] Mode),
    #[rune(constructor)]
    FocusUp,
    #[rune(constructor)]
    FocusDown,
    #[rune(constructor)]
    FocusLeft,
    #[rune(constructor)]
    FocusRight,
    #[rune(constructor)]
    FocusPrevious,
    #[rune(constructor)]
    FocusCommandLine,
    #[rune(constructor)]
    JoinLines,
    #[rune(constructor)]
    NewlineAbove,
    #[rune(constructor)]
    NewlineBelow,
    #[rune(constructor)]
    DeleteForwards,
    #[rune(constructor)]
    DeleteBackwards,
    #[rune(constructor)]
    Sequence(#[rune(get)] Vec<Command>),
}

#[derive(Any, Clone, Debug, Eq, PartialEq)]
pub enum Direction {
    #[rune(constructor)]
    Up,
    #[rune(constructor)]
    Down,
    #[rune(constructor)]
    Left,
    #[rune(constructor)]
    Right,
}
