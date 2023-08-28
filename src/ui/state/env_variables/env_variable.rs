use anyhow::{bail, Error};
use derive_more::AsRef;
use parse_display::Display;
use std::str;

/// Environment variable name that can be (un)set by user and is set in
/// subprocesses.
#[derive(Debug, AsRef, Clone, Display, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[display("{0}")]
pub struct EnvVariable(String);

impl str::FromStr for EnvVariable {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().any(|c| c.is_whitespace() || c.is_uppercase()) {
            bail!(
                "Failed to parse environment variable name '{}', neither whitespace nor uppercase characters are allowed.",
                s
            );
        }
        Ok(Self(s.to_owned()))
    }
}
