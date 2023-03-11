#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Mode {
    Insert,
    Visual { block_mode: bool },
    Normal,
}
