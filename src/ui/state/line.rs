use tui::{
	style::Style,
	widgets::{Cell, Row},
};

pub struct Line {
	line: String,
	style: Style,
}

impl Line {
	pub fn new(line: String, style: Style) -> Self {
		Self { line, style }
	}

	pub fn draw(&self, selected: Style) -> Row {
		Row::new(vec![
			Cell::from(" ").style(selected),
			Cell::from(" ".to_owned() + &self.line).style(self.style),
		])
	}

	pub fn set_style(&mut self, style: Style) {
		self.style = style;
	}
}
