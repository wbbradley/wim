use crate::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Target {
    View(ViewKey),
    Named(String),
    Previous,
    Focused,
    ViewMap,
    Root, // Should be the editor.
}

impl std::str::FromStr for Target {
    type Err = crate::error::Error;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Err(error!("not impl yet"))
    }
}
