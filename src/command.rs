use crate::key::Key;
use crate::mode::Mode;
use crate::noun::Noun;
use crate::propvalue::PropertyValue;
use crate::rel::Rel;
use crate::view::ViewKey;
use rune::Any;

#[derive(Any, Clone, Debug)]
pub enum Command {
    #[rune(constructor)]
    Open {
        #[rune(get)]
        filename: String,
    },
    #[rune(constructor)]
    Save,
    #[rune(constructor)]
    Call {
        #[rune(get)]
        name: String,
        #[rune(get)]
        args: Vec<CallArg>,
    },
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
    FocusViewKey(#[rune(get)] String),
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
    #[rune(constructor)]
    Key(#[rune(get)] Key),
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

#[derive(Any, Clone, Debug)]
pub enum CallArg {
    Ref(ViewKey, String),
    Val(PropertyValue),
}
