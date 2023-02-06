use tui::{
	style::Style,
	widgets::{Cell, Row},
};

pub struct Line {
	unformatted: String,
	// TODO: memory improvement: make optional only use with field selection
	formatted: Option<String>,
	style: Style,
}

impl Line {
	pub fn new(unformatted: String, formatted: Option<String>, style: Style) -> Self {
		Self {
			unformatted,
			formatted,
			style,
		}
	}

	pub fn draw(&self, selected: Style) -> Row {
		// TODO: fix with shorter syntax
		let line = match &self.formatted {
			Some(formatted) => formatted,
			None => &self.unformatted,
		};
		// let line = self.formatted.unwrap_or(&self.unformatted);
		Row::new(vec![
			Cell::from(" ").style(selected),
			Cell::from(" ".to_owned() + line).style(self.style),
		])
	}

	pub fn update_style(&mut self, style: Style) {
		self.style = style;
	}

	pub fn unformatted(&self) -> &String {
		&self.unformatted
	}
}
