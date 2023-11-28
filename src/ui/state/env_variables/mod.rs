mod env_variable;

use anyhow::{bail, Result};
use itertools::Itertools;
use std::{collections::HashMap, fmt, io::Write};
use tabwriter::TabWriter;

use crate::{
    config::{Operation, OperationParsed, Operations, OperationsParsed},
    utils::command::CommandBuilder,
};

pub use self::env_variable::EnvVariable;

#[derive(Default, Debug)]
pub struct EnvVariables(HashMap<EnvVariable, String>);

impl EnvVariables {
    /// Create a new empty structure of environment variables.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Receive parsed operations, but only execute the "set-env" operations.
    pub async fn generate_initial(value: OperationsParsed) -> Result<Self> {
        // TODO: consider trying to use async iterators to do this in one iterator pass (instead of the mut hashmap) once stable
        let mut map = HashMap::new();
        for op in value.into_iter() {
            match op {
                OperationParsed::SetEnv(env_variable, command) => {
                    let output = CommandBuilder::new(command)
                        .blocking()
                        .with_output()
                        .execute()
                        .await?;
                    map.insert(env_variable, output);
                }
                other_op => bail!("Invalid operation for env variable setup (only \"set-env\" operations allowed here): {}", other_op),
            }
        }
        Ok(Self(map))
    }

    /// Receive parsed operations, but only execute the "set-env" operations.
    pub async fn generate_initial2(&mut self, operations: Operations) -> Result<()> {
        // TODO: consider trying to use async iterators to do this in one iterator pass (instead of the mut hashmap) once stable
        for op in operations.into_iter() {
            match op {
                Operation::SetEnv(env_variable, blocking_cmd) => {
                    log::info!("executing cmd");

                    let cmd_output = blocking_cmd.execute().await?;

                    log::info!("finished executing cmd");

                    self.0.insert(env_variable, cmd_output);
                }
                // other_op => bail!("Invalid operation for env variable setup (only \"set-env\" operations allowed here): {}", other_op),
                _ => bail!("Only \"set-env\" operations allowed here."),
            }
        }
        Ok(())
    }

    pub fn merge_new_envs(&mut self, env_variables: Self) {
        self.0.extend(env_variables.0);
    }

    // TODO: expose EnvVariableValue type instead of String

    /// Add an environment variable mapping.
    pub fn set_env(&mut self, env_var: EnvVariable, value: String) {
        self.0.insert(env_var, value);
    }

    /// Unset/remove the specified environment variable.
    pub fn unset_env(&mut self, env_var: &EnvVariable) {
        self.0.remove(env_var);
    }

    /// Write formatted version (insert elastic tabstops) to a buffer.
    fn write<W: Write>(&self, writer: W) -> Result<()> {
        let mut tw = TabWriter::new(writer);
        for (env_variable, value) in self.0.iter().sorted() {
            writeln!(tw, "{}\t= \"{}\"", env_variable, value)?;
        }
        tw.flush()?;
        Ok(())
    }

    // TODO: code duplication from KeybindingsParsed
    fn fmt(&self) -> Result<String> {
        let mut buffer = vec![];
        self.write(&mut buffer)?;
        let written = String::from_utf8(buffer)?;
        Ok(written)
    }
}

// TODO: code duplication from KeybindingsParsed
impl fmt::Display for EnvVariables {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = self.fmt().map_err(|_| fmt::Error)?;
        f.write_str(&formatted)
    }
}

// TODO: maybe find solution where we don't have to clone env_variables everytime
impl From<&EnvVariables> for HashMap<String, String> {
    fn from(value: &EnvVariables) -> Self {
        value
            .0
            .iter()
            .map(|(env_var, str)| (env_var.as_ref().to_owned(), str.to_owned()))
            .collect()
    }
}

// TODO: seems like boilerplate
impl FromIterator<(EnvVariable, String)> for EnvVariables {
    fn from_iter<I: IntoIterator<Item = (EnvVariable, String)>>(iter: I) -> Self {
        EnvVariables(iter.into_iter().collect())
    }
}
