#[allow(dead_code)]
#[derive(Debug)]
pub enum Mode {
    Insert,
    Visual { block: bool },
    Command,
}
