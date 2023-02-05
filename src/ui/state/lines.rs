use super::Line;
use anyhow::Result;
use derive_more::Deref;
use std::io::Write;
use tabwriter::TabWriter;
use tui::style::Style;

#[derive(Deref)]
pub struct Lines {
	// TODO: if i'm keeping both formatted and unformatted anyways, might as well integrate into Line itself and only have one vector
	formatted: Vec<Line>,
	// TODO: memory improvement: make optional only use with field selection
	#[deref]
	unformatted: Vec<String>,
	field_seperator: Option<String>,
	style: Style,
}

impl Lines {
	pub fn new(field_seperator: Option<String>, style: Style) -> Self {
		Self {
			formatted: vec![],
			unformatted: vec![],
			field_seperator,
			style,
		}
	}

	pub fn update(&mut self, lines: String) -> Result<()> {
		self.unformatted = lines.lines().map(str::to_owned).collect();

		self.formatted = match &self.field_seperator {
			Some(seperator) => {
				// TODO: cleaner syntax
				let mut tw = TabWriter::new(vec![]);
				write!(&mut tw, "{}", lines.replace(seperator, "\t"))?;
				tw.flush()?;
				String::from_utf8(tw.into_inner()?)?
			}
			None => lines,
		}
		.lines()
		.map(|line| Line::new(line.to_owned(), self.style))
		.collect();

		Ok(())
	}

	pub fn update_style(&mut self, index: usize, new_style: Style) {
		if let Some(line) = self.formatted.get_mut(index) {
			line.set_style(new_style);
		}
	}

	pub fn formatted(&self) -> std::slice::Iter<Line> {
		self.formatted.iter()
	}

	pub fn unformatted(&self) -> std::slice::Iter<String> {
		self.unformatted.iter()
	}

	pub fn get_unformatted(&self, index: usize) -> Option<String> {
		self.unformatted.get(index).map(String::to_owned)
	}
}
