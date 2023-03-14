use rune::Any;

#[derive(Any, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Noun {
    #[rune(constructor)]
    Char,
    #[rune(constructor)]
    Word,
    #[rune(constructor)]
    Line,
}
