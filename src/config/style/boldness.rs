use clap::ValueEnum;
use parse_display::{Display, FromStr};
use serde::Deserialize;
use strum::EnumIter;

/// A wrapper around ratatui's `Modifier::BOLD`.
#[derive(Deserialize, FromStr, Display, Clone, Default, ValueEnum, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[display(style = "kebab-case")]
pub enum Boldness {
    Bold,
    NonBold,
    /// Don't enforce any specific style.
    #[default]
    Unspecified,
}

impl Boldness {
    /// Returns `other` if self is `Unspecified`, otherwise returns self.
    pub fn or(self, other: Self) -> Self {
        match self {
            Boldness::Unspecified => other,
            boldness => boldness,
        }
    }
}
