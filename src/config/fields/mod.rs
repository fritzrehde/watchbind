mod field_selection;
mod field_separator;

use anyhow::{bail, Result};
use itertools::Itertools;
use std::io::Write;
use tabwriter::TabWriter;

pub use self::field_selection::FieldSelections;
pub use self::field_separator::FieldSeparator;

/// Any string line can be seen as a sequence of fields, separated (or
/// delimited) by a field separator. Only fields that are selected will
/// be displayed.
pub struct Fields {
    separator: Option<FieldSeparator>,
    selections: Option<FieldSelections>,
}

impl Fields {
    pub fn try_new(
        separator: Option<FieldSeparator>,
        selections: Option<FieldSelections>,
    ) -> Result<Self> {
        if selections.is_some() && separator.is_none() {
            bail!("Cannot specify/apply field selections without specifying a field separator");
        }
        Ok(Self {
            separator,
            selections,
        })
    }
}

/// Format a string as a table that has its fields separated by an elastic
/// tabstop, and only displays the fields that should be selected.
/// Only applies any formatting if a separator or selection is present.
pub trait TableFormatter {
    fn format_as_table(&self, fields: &Fields) -> Result<Option<String>>;
}

impl TableFormatter for &str {
    fn format_as_table(&self, fields: &Fields) -> Result<Option<String>> {
        let table = match &fields.separator {
            Some(separator) => {
                let separator = separator.as_ref();

                let formatted_lines = match &fields.selections {
                    Some(selections) => self
                        .lines()
                        .map(|line| {
                            line.split(separator)
                                .enumerate()
                                // TODO: seems inefficient, try applying selection to whole line at a time
                                .filter_map(|(idx, field)| {
                                    selections.contains(idx).then_some(field)
                                })
                                .join("\t")
                        })
                        .join("\n"),
                    None => self.replace(separator, "\t"),
                };

                let mut tw = TabWriter::new(vec![]);
                write!(tw, "{}", formatted_lines)?;
                tw.flush()?;

                let table = String::from_utf8(tw.into_inner()?)?;
                Some(table)
            }
            None => None,
        };
        Ok(table)
    }
}
