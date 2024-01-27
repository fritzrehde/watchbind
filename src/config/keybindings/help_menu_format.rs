use anyhow::{Error, Result};
use derive_more::IntoIterator;
use parse_display::{Display, FromStr};
use serde::Deserialize;
use std::str;

/// Specifies which columns should be included in the keybindings help menu,
/// and in what order.
#[derive(Debug, Deserialize, Clone, IntoIterator)]
#[cfg_attr(test, derive(PartialEq))]
pub struct KeybindingsHelpMenuFormat(#[into_iterator(ref)] Vec<KeybindingsHelpMenuColumn>);

#[derive(Debug, Deserialize, FromStr, Display, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "kebab-case")]
#[display(style = "kebab-case")]
pub enum KeybindingsHelpMenuColumn {
    Key,
    Operations,
    Description,
}

// TODO: should get generated by e.g. parse_display directly
impl str::FromStr for KeybindingsHelpMenuFormat {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let help_menu_columns = s
            .split(',')
            .map(KeybindingsHelpMenuColumn::from_str)
            .collect::<Result<_, _>>()?;
        Ok(Self(help_menu_columns))
    }
}