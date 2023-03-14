use rune::Any;

#[derive(Any, Copy, Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Rel {
    #[rune(constructor)]
    Prior,
    #[rune(constructor)]
    Beginning,
    #[rune(constructor)]
    End,
    #[rune(constructor)]
    Next,
}
