#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Noun {
    Char,
    Word,
    Line,
}

impl std::str::FromStr for Noun {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "char" => Ok(Noun::Char),
            "word" => Ok(Noun::Word),
            "line" => Ok(Noun::Line),
            missing => Err(Self::Err::new(format!("{} is not a valid Noun", missing))),
        }
    }
}
