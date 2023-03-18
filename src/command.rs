use crate::dk::{ToDK, DK};
use crate::prelude::*;
use crate::types::Pos;
use crate::view::ViewKey;
use rune::Any;

#[derive(Any, Clone, Debug)]
pub struct Command {
    #[rune(get)]
    name: String,
    #[rune(get)]
    args: Vec<CallArg>,
}

impl ToDK for Command {
    fn with_view_key(self, view_key: ViewKey) -> DK {
        DK::DK(view_key, Form::Command(self))
    }
}

pub fn make_command<T>(name: T, args: Vec<CallArg>) -> Command
where
    T: Into<String>,
{
    Command {
        name: name.into(),
        args: args.into_iter().map(|a| a).collect(),
    }
}

macro_rules! command {
    ($name:expr) => {{
        $crate::command::make_command($name, Default::default())
    }};
    ($name:expr, $($args:expr),+) => {{
        $crate::command::make_command($name, vec![$($args),+])
    }};
}
pub(crate) use command;

#[derive(Any, Clone, Debug)]
pub enum CallArg {
    #[rune(constructor)]
    Ref(#[rune(get)] ViewKey, #[rune(get)] String),
    #[rune(constructor)]
    Int(#[rune(get)] i64),
    #[rune(constructor)]
    ViewKey(#[rune(get)] ViewKey),
    #[rune(constructor)]
    Float(#[rune(get)] f64),
    #[rune(constructor)]
    String(#[rune(get)] String),
    #[rune(constructor)]
    Bool(#[rune(get)] bool),
    #[rune(constructor)]
    Pos(#[rune(get)] Pos),
}
