mod operation;

use anyhow::{Context, Result};
use derive_more::{From, IntoIterator};
use itertools::Itertools;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ui::EnvVariables;

pub use self::operation::{Operation, OperationParsed};

#[derive(IntoIterator, From)]
pub struct Operations(#[into_iterator(ref)] Vec<Operation>);

impl Operations {
    pub fn from_parsed(
        operations_parsed: OperationsParsed,
        env_variables: &Arc<Mutex<EnvVariables>>,
    ) -> Self {
        Self(
            operations_parsed
                .0
                .into_iter()
                .map(|op| Operation::from_parsed(op, env_variables))
                .collect(),
        )
    }

    // TODO: find crate that removes this boilerplate
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, IntoIterator, Eq, Ord, PartialEq, PartialOrd, From, Clone)]
pub struct OperationsParsed(#[into_iterator(ref)] Vec<OperationParsed>);

impl TryFrom<Vec<String>> for OperationsParsed {
    type Error = anyhow::Error;
    fn try_from(vec: Vec<String>) -> Result<Self, Self::Error> {
        let operations = vec
            .into_iter()
            .map(|op| {
                // TODO: error messages from the parsing of attributes inside an operation (e.g. an EnvVariable) is not displayed here
                op.parse()
                    .with_context(|| format!("Failed to parse operation: {}", op))
                    .map_err(anyhow::Error::from)
            })
            .collect::<Result<_>>()?;
        Ok(Self(operations))
    }
}

// TODO: find cleaner/less boilerplate way using special crate, looks similar to toml format, maybe serialize into that
impl std::fmt::Display for OperationsParsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: we mark the boundaries of the string with " chars, but those are also contained inside the operations, making it inconsistent
        let formatted_operations = self.0.iter().map(|op| format!("\"{}\"", op)).join(", ");
        write!(f, "[ {} ]", formatted_operations)
    }
}
