#[derive(Debug)]
pub enum Command {
    Open { filename: String },
    Save,
    Move(Direction),
}

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
