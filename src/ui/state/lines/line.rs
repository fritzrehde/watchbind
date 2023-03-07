use tui::{style::Style, widgets::Cell};

pub struct Line {
	unformatted: String,
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

	pub fn draw(&self) -> Cell {
		let line = self.formatted.as_ref().unwrap_or(&self.unformatted);
		Cell::from(" ".to_owned() + line).style(self.style)
	}

	pub fn update_style(&mut self, style: Style) {
		self.style = style;
	}

	pub fn unformatted(&self) -> &String {
		&self.unformatted
	}
}
