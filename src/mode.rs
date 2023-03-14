use rune::Any;

#[allow(dead_code)]
#[derive(Any, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    #[rune(constructor)]
    Insert,
    #[rune(constructor)]
    Visual {
        #[rune(get)]
        block_mode: bool,
    },
    #[rune(constructor)]
    Normal,
}
