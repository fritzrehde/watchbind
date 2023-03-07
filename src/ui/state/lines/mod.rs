mod line;

pub use line::Line;

use crate::config::Styles;
use anyhow::Result;
use derive_more::IntoIterator;
use itertools::izip;
use std::io::Write;
use tabwriter::TabWriter;
use tui::style::Style;

#[derive(IntoIterator)]
pub struct Lines {
	#[into_iterator(ref)]
	lines: Vec<Line>,
	field_separator: Option<String>,
	styles: Styles,
	header_lines: usize,
}

impl Lines {
	pub fn new(field_separator: Option<String>, styles: Styles, header_lines: usize) -> Self {
		Self {
			lines: vec![],
			field_separator,
			styles,
			header_lines,
		}
	}

	pub fn update(&mut self, lines: String) -> Result<()> {
		let formatted: Vec<Option<String>> = match &self.field_separator {
			Some(separator) => {
				// TODO: cleaner syntax
				let mut tw = TabWriter::new(vec![]);
				write!(&mut tw, "{}", lines.replace(separator, "\t"))?;
				tw.flush()?;

				String::from_utf8(tw.into_inner()?)?
					.lines()
					.map(|line| Some(line.to_owned()))
					.collect()
			}
			None => lines.lines().map(|_| None).collect(),
		};

		self.lines = izip!(lines.lines(), formatted)
			.enumerate()
			.map(|(i, (unformatted, formatted))| {
				let style = if i < self.header_lines {
					self.styles.header
				} else {
					self.styles.line
				};

				Line::new(unformatted.to_owned(), formatted, style)
			})
			.collect();

		Ok(())
	}

	pub fn update_style(&mut self, index: usize, new_style: Style) {
		if let Some(line) = self.lines.get_mut(index) {
			line.update_style(new_style);
		}
	}

	pub fn unformatted(&self) -> Vec<&String> {
		self.lines.iter().map(Line::unformatted).collect()
	}

	pub fn get_unformatted(&self, index: usize) -> Option<String> {
		self.lines.get(index).map(|line| line.unformatted().clone())
	}

	pub fn len(&self) -> usize {
		self.lines.len()
	}

	pub fn is_empty(&self) -> bool {
		self.lines.is_empty()
	}
}
