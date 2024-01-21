mod key;
mod operations;

use anyhow::{bail, Context, Result};
use derive_more::From;
use itertools::Itertools;
use serde::Deserialize;
use std::io::Write;
use std::sync::Arc;
use std::{collections::HashMap, fmt};
use tabwriter::TabWriter;
use tokio::sync::Mutex;

use crate::ui::EnvVariables;

pub use self::key::{KeyCode, KeyEvent, KeyModifier};
pub use self::operations::{OperationExecutable, OperationParsed, Operations, OperationsParsed};

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

#[derive(Debug, Clone, PartialEq, Eq, From)]
pub struct KeybindingsParsed(HashMap<KeyEvent, OperationsParsed>);

impl KeybindingsParsed {
    /// Merge two keybinding hashmaps, where a value is taken from `opt_a` over
    /// `opt_b` on identical keys.
    pub fn merge(opt_a: Option<Self>, opt_b: Option<Self>) -> Option<Self> {
        match opt_a {
            Some(a) => match opt_b {
                Some(b) => {
                    // If `a` and `b` have same key => keep `a`'s value
                    let mut merged = b.0;
                    merged.extend(a.0);
                    Some(Self(merged))
                }
                None => Some(a),
            },
            None => opt_b,
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_keybindings() {
        let k1 = KeyEvent::new(KeyModifier::None, KeyCode::BackTab);
        let k2 = KeyEvent::new(KeyModifier::None, KeyCode::Backspace);
        let k3 = KeyEvent::new(KeyModifier::None, KeyCode::Delete);

        let v1 = OperationsParsed::from(vec![OperationParsed::ExecuteBlocking("v1".to_string())]);
        let v2 = OperationsParsed::from(vec![OperationParsed::ExecuteBlocking("v2".to_string())]);
        let v3 = OperationsParsed::from(vec![OperationParsed::ExecuteBlocking("v3".to_string())]);
        let v4 = OperationsParsed::from(vec![OperationParsed::ExecuteBlocking("v4".to_string())]);

        let a: KeybindingsParsed = HashMap::from([(k1.clone(), v1), (k3.clone(), v4)]).into();
        let b: KeybindingsParsed = HashMap::from([(k1.clone(), v2), (k2.clone(), v3)]).into();

        let merged = KeybindingsParsed::merge(Some(a.clone()), Some(b.clone()))
            .expect("merge should not be empty given both inputs are some");

        // Assert that values from `a` were prioritized over those from `b`.

        // If both `a` and `b` contain `k1`, check that `a`'s value was used.
        assert!(
            a.0.contains_key(&k1) && b.0.contains_key(&k1),
            "both a and b should contain k1"
        );
        assert_ne!(
            a.0.get(&k1),
            b.0.get(&k1),
            "a and b should contain different values for k1"
        );
        assert_eq!(a.0.get(&k1), merged.0.get(&k1), "a's value should be used");

        // If only `b` contains `k2` (and `a` does not), check that `b`'s
        // value was used.
        assert!(
            b.0.contains_key(&k2) && !a.0.contains_key(&k2),
            "only b should contain k2, a should not"
        );
        assert_eq!(b.0.get(&k2), merged.0.get(&k2), "b's value should be used");

        // If only `a` contains `k3` (and `b` does not), check that `a`'s
        // value was used.
        assert!(
            a.0.contains_key(&k3) && !b.0.contains_key(&k3),
            "only a should contain k3, b should not"
        );
        assert_eq!(a.0.get(&k3), merged.0.get(&k3), "a's value should be used");
    }
}
