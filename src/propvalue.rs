use crate::types::Pos;
use rune::Any;

#[derive(Any, Clone, Debug)]
pub enum PropertyValue {
    #[rune(constructor)]
    Int(#[rune(get)] i64),
    #[rune(constructor)]
    Float(#[rune(get)] f64),
    #[rune(constructor)]
    String(#[rune(get)] String),
    #[rune(constructor)]
    Bool(#[rune(get)] bool),
    #[rune(constructor)]
    Pos(#[rune(get)] Pos),
}
