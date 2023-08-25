mod operation;

pub use operation::Operation;

use anyhow::{Context, Result};
use derive_more::{From, IntoIterator};
use itertools::Itertools;

#[derive(Clone, IntoIterator, Eq, Ord, PartialEq, PartialOrd, From)]
pub struct Operations(#[into_iterator(ref)] Vec<Operation>);

impl TryFrom<Vec<String>> for Operations {
    type Error = anyhow::Error;
    fn try_from(vec: Vec<String>) -> Result<Self, Self::Error> {
        let operations = vec
            .into_iter()
            .map(|op| {
                op.parse()
                    .with_context(|| format!("Failed to parse operation: {}", op))
                    .map_err(anyhow::Error::from)
            })
            .collect::<Result<_>>()?;
        Ok(Self(operations))
    }
}

// TODO: find cleaner/less boilerplate way using special crate, looks similar to toml format, maybe serialize into that
impl std::fmt::Display for Operations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted_operations = self.0.iter().map(|op| format!("\"{}\"", op)).join(", ");
        write!(f, "[ {} ]", formatted_operations)
    }
}

impl Operations {
    // TODO: find crate that removes this boilerplate
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
