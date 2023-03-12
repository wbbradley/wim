#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    Insert,
    Visual { block_mode: bool },
    Normal,
}
