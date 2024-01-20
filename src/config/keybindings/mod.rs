mod key;
mod operations;

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use serde::Deserialize;
use std::io::Write;
use std::sync::Arc;
use std::{collections::HashMap, fmt};
use tabwriter::TabWriter;
use tokio::sync::Mutex;

use crate::ui::EnvVariables;

pub use self::key::{KeyCode, KeyEvent, KeyModifier};
pub use self::operations::{Operation, OperationParsed, Operations, OperationsParsed};

pub struct Keybindings(HashMap<KeyEvent, Operations>);

impl Keybindings {
    pub fn get_operations(&self, key: &KeyEvent) -> Option<&Operations> {
        self.0.get(key)
    }

    pub fn from_parsed(
        keybindings_parsed: KeybindingsParsed,
        env_variables: &Arc<Mutex<EnvVariables>>,
    ) -> Self {
        Self(
            keybindings_parsed
                .0
                .into_iter()
                .map(|(key, ops)| (key, Operations::from_parsed(ops, env_variables)))
                .collect(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeybindingsParsed(HashMap<KeyEvent, OperationsParsed>);

impl KeybindingsParsed {
    /// Write formatted version (insert elastic tabstops) to a buffer.
    fn write<W: Write>(&self, writer: W) -> Result<()> {
        let mut tw = TabWriter::new(writer);
        for (key, operations) in self.0.iter().sorted() {
            writeln!(tw, "{}\t= {}", key, operations)?;
        }
        tw.flush()?;
        Ok(())
    }

    fn fmt(&self) -> Result<String> {
        let mut buffer = vec![];
        self.write(&mut buffer)?;
        let written = String::from_utf8(buffer)?;
        Ok(written)
    }
}

impl fmt::Display for KeybindingsParsed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = self.fmt().map_err(|_| fmt::Error)?;
        f.write_str(&formatted)
    }
}

impl TryFrom<StringKeybindings> for KeybindingsParsed {
    type Error = anyhow::Error;
    fn try_from(value: StringKeybindings) -> Result<Self, Self::Error> {
        let keybindings = value
            .0
            .into_iter()
            .map(|(key, ops)| {
                Ok((
                    key.parse()
                        .with_context(|| format!("Invalid KeyEvent: {}", key))?,
                    ops.try_into()?,
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Self(keybindings))
    }
}

// TODO: remove once clap supports parsing directly into HashMap
pub type ClapKeybindings = Vec<(String, Vec<String>)>;

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct StringKeybindings(HashMap<String, Vec<String>>);

impl KeybindingsParsed {
    /// Merge two keybinding hashmaps, where a value is taken from `opt_a` over
    /// `opt_b` on identical keys.
    pub fn merge(opt_a: Option<Self>, opt_b: Option<Self>) -> Option<Self> {
        match opt_a {
            Some(a) => match opt_b {
                Some(b) => {
                    // `a` and `b` have same key => keep `a`'s value
                    let mut merged = b.0;
                    merged.extend(a.0);
                    Some(Self(merged))
                }
                None => Some(a),
            },
            None => opt_b,
        }
    }
}

impl From<ClapKeybindings> for StringKeybindings {
    fn from(clap: ClapKeybindings) -> Self {
        Self(clap.into_iter().collect())
    }
}

// TODO: implement FromStr trait
// TODO: replace with nom
// TODO: parse to Vec<Keybinding> and provide from_str for keybinding
pub fn parse_str(s: &str) -> Result<(String, Vec<String>)> {
    let Some((key, operations)) = s.split_once(':') else {
        bail!("invalid format: expected \"KEY:OP[+OP]*\", found \"{}\"", s);
    };

    Ok((
        key.to_string(),
        operations
            .split('+')
            .map(|op| op.trim().to_owned())
            .collect(),
    ))
}
