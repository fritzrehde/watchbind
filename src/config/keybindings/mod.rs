mod key;
mod operations;

pub use key::KeyEvent;
pub use operations::{Operation, Operations};

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use serde::Deserialize;
use std::io::Write;
use std::{collections::HashMap, fmt};
use tabwriter::TabWriter;

#[derive(Clone)]
pub struct Keybindings(HashMap<KeyEvent, Operations>);

impl Keybindings {
    pub fn get_operations(&self, key: &KeyEvent) -> Option<&Operations> {
        self.0.get(key)
    }

    /// Write formatted version (insert elastic tabstops) to a buffer.
    fn write<W: Write>(&self, writer: W) -> Result<()> {
        let mut tw = TabWriter::new(writer);
        for (key, operations) in self.0.iter().sorted() {
            writeln!(tw, "{}:\t{}", key, operations)?;
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

impl fmt::Display for Keybindings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = self.fmt().map_err(|_| fmt::Error)?;
        f.write_str(&formatted)
    }
}

impl TryFrom<StringKeybindings> for Keybindings {
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

#[derive(Deserialize)]
pub struct StringKeybindings(HashMap<String, Vec<String>>);

impl StringKeybindings {
    pub fn merge(new_opt: Option<Self>, old_opt: Option<Self>) -> Option<Self> {
        match new_opt {
            Some(new) => match old_opt {
                Some(old) => {
                    // new and old have same key => keep new value
                    let mut merged = old.0;
                    merged.extend(new.0);
                    Some(StringKeybindings(merged))
                }
                None => Some(new),
            },
            None => old_opt,
        }
    }
}

impl From<ClapKeybindings> for StringKeybindings {
    fn from(clap: ClapKeybindings) -> Self {
        Self(clap.into_iter().collect())
    }
}

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
