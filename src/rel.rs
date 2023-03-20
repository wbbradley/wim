#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Rel {
    Prior,
    Begin,
    End,
    Next,
}

impl std::str::FromStr for Rel {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "prior" => Ok(Rel::Prior),
            "begin" => Ok(Rel::Begin),
            "end" => Ok(Rel::End),
            "next" => Ok(Rel::Next),
            missing => Err(Self::Err::new(format!("{} is not a valid Rel", missing))),
        }
    }
}
