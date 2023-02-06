use super::Line;
use anyhow::Result;
use derive_more::Deref;
use itertools::izip;
use std::io::Write;
use tabwriter::TabWriter;
use tui::style::Style;

#[derive(Deref)]
pub struct Lines {
	#[deref]
	lines: Vec<Line>,
	field_seperator: Option<String>,
	style: Style,
}

impl Lines {
	pub fn new(field_seperator: Option<String>, style: Style) -> Self {
		Self {
			lines: vec![],
			field_seperator,
			style,
		}
	}

	pub fn update(&mut self, lines: String) -> Result<()> {
		// TODO: merge into one iteration with izip
		self.lines = lines
			.lines()
			.map(|line| Line::new(line.to_owned(), self.style))
			.collect();

		if let Some(seperator) = &self.field_seperator {
			// TODO: cleaner syntax
			let mut tw = TabWriter::new(vec![]);
			write!(&mut tw, "{}", lines.replace(seperator, "\t"))?;
			tw.flush()?;

			izip!(
				self.lines.iter_mut(),
				String::from_utf8(tw.into_inner()?)?.lines(),
			)
			.for_each(|(line, formatted)| line.format(formatted.to_owned()));
		}

		Ok(())
	}

	pub fn update_style(&mut self, index: usize, new_style: Style) {
		if let Some(line) = self.lines.get_mut(index) {
			line.set_style(new_style);
		}
	}

	// TODO: use iter instead of vec
	pub fn unformatted(&self) -> Vec<&String> {
		self.lines.iter().map(Line::unformatted).collect()
	}

	pub fn get_unformatted(&self, index: usize) -> Option<String> {
		self.lines.get(index).map(|line| line.unformatted().clone())
	}
}
