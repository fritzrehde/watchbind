use anyhow::Result;
use derive_more::AsRef;
use parse_display::FromStr;
use serde::Deserialize;
use std::io::Write;
use tabwriter::TabWriter;

// TODO: could also be char, but that makes it more restrictive
#[derive(Debug, Deserialize, FromStr, Clone, AsRef)]
#[cfg_attr(test, derive(PartialEq))]
pub struct FieldSeparator(String);

impl FieldSeparator {
    /// Formats a string as a table by replacing all field separators
    /// with elastic tabstops.
    pub fn format_string_as_table(&self, s: &str) -> Result<String> {
        let separator_replaced = s.replace(&self.0, "\t");

        let mut tw = TabWriter::new(vec![]);
        write!(tw, "{}", separator_replaced)?;
        tw.flush()?;

        let table = String::from_utf8(tw.into_inner()?)?;
        Ok(table)
    }
}
