use crate::color::{BgColor, FgColor};
use crate::error::{ErrorContext, Result};
use serde_derive::Deserialize;
use std::fs;

// Top level struct to hold the TOML data.
#[derive(Deserialize)]
pub struct Settings {
    pub display: DisplaySettings,
}

#[derive(Deserialize)]
pub struct DisplaySettings {
    pub fg: FgColor,
    pub bg: BgColor,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let filename = "settings.toml";
        let contents = fs::read_to_string(filename)?;
        toml::from_str(&contents).context("load-settings")
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            display: DisplaySettings {
                fg: FgColor::White,
                bg: BgColor::Black,
            },
        }
    }
}
