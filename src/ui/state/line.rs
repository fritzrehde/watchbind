use tui::{
	style::Style,
	widgets::{Cell, Row},
};

pub struct Line {
	raw: String,
	// formatted: String,
	style: Style,
}

impl Line {
	pub fn new(raw: String, style: Style) -> Self {
		Self {
			raw: raw.clone(),
			// formatted: raw,
			style,
		}
	}

	pub fn draw(&self, selected: Style) -> Row {
		Row::new(vec![
			Cell::from(" ").style(selected),
			Cell::from(" ".to_owned() + &self.raw).style(self.style),
		])
	}

	pub fn set_style(&mut self, style: Style) {
		self.style = style;
	}

	// TODO: rename
	pub fn get(&self) -> String {
		self.raw.clone()
	}
}
