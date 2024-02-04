mod env_variable;

use std::collections::HashMap;

use crate::config::Table;

pub use self::env_variable::EnvVariable;

#[derive(Default, Debug, Clone)]
pub struct EnvVariables(HashMap<EnvVariable, String>);

impl EnvVariables {
    /// Create a new empty structure of environment variables.
    pub fn new() -> Self {
        Self(HashMap::new())
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

    pub fn display<U>(&self, display_width: U) -> String
    where
        usize: From<U>,
    {
        let column_names = &["environment variable".to_string(), "value".to_string()];

        let rows_iter = self
            .0
            .iter()
            .map(|(env_variable, value)| [env_variable.to_string(), value.to_owned()]);

        Table::new(rows_iter)
            .width(Some(display_width))
            .left_margin(2)
            .header(column_names)
            .border()
            .make_string()
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
