#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    Insert,
    Visual(VisualMode),
    Normal,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VisualMode {
    Char,
    Line,
    Block,
}

impl std::str::FromStr for Mode {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "insert" => Ok(Mode::Insert),
            "visual" => Ok(Mode::Visual(VisualMode::Char)),
            "visual-line" => Ok(Mode::Visual(VisualMode::Line)),
            "visual-block" => Ok(Mode::Visual(VisualMode::Block)),
            missing => Err(Self::Err::new(format!(
                "{} is not a valid editing Mode",
                missing
            ))),
        }
    }
}
