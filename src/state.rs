use crate::style::Styles;
use itertools::izip;
use tui::{
	backend::Backend,
	layout::Constraint,
	style::Style,
	widgets::{Cell, Row, Table},
	Frame,
};

const FIRST_INDEX: usize = 0;

type Line = (String, Style);

pub struct State {
	lines: Vec<Line>,
	selected: Vec<bool>,
	styles: Styles,
	cursor: Option<usize>,
}

impl State {
	pub fn new(styles: &Styles) -> State {
		State {
			lines: vec![],
			selected: vec![],
			styles: *styles,
			cursor: None,
		}
	}

	pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
		let rows: Vec<Row> = izip!(self.lines.iter(), self.selected.iter())
			.map(|((line, style), &selected)| {
				Row::new(vec![
					Cell::from(" ").style(if selected {
						self.styles.selected
					} else {
						self.styles.line
					}),
					Cell::from(" ".to_owned() + &line).style(*style)
				])
			})
			.collect();

		let table = Table::new(rows)
			.widths(&[Constraint::Length(1), Constraint::Percentage(100)])
			.column_spacing(0);

		frame.render_widget(table, frame.size());
	}

	pub fn set_lines(&mut self, lines: Vec<String>) {
		self.selected.resize(lines.len(), false);
		self.lines = lines.into_iter().map(|line| (line, self.styles.line)).collect();
		self.cursor_calibrate();
	}

	fn cursor_calibrate(&mut self) {
		match self.cursor_position() {
			None => self.first(),
			Some(i) => self.cursor_move(i as isize),
		};
	}

	fn cursor_position(&mut self) -> Option<usize> {
		self.cursor
	}

	fn cursor_move(&mut self, index: isize) {
		let old = self.cursor_position();
		let new = match self.lines.is_empty() {
			true => None,
			false => {
				let first = FIRST_INDEX as isize;
				let last = self.last_index() as isize;
				Some(index.clamp(first, last) as usize)
			}
		};

		self.cursor = new;
		self.cursor_adjust_style(old, new);
	}

	fn cursor_adjust_style(&mut self, old: Option<usize>, new: Option<usize>) {
		if let Some(old_index) = old {
			self.lines[old_index].1 = self.styles.line;
		}
		if let Some(new_index) = new {
			self.lines[new_index].1 = self.styles.cursor;
		}
	}

	fn get_cursor_line(&mut self) -> String {
		if let Some(i) = self.cursor_position() {
			if let Some((line, _)) = self.lines.get(i) {
				return line.clone();
			}
		}
		// no line selected => LINE=""
		"".to_string()
	}

	pub fn get_selected_lines(&mut self) -> String {
		let lines: String = izip!(self.lines.iter(), self.selected.iter())
			.filter_map(
				|((line, _), &selected)| {
					if selected {
						Some(line.clone())
					} else {
						None
					}
				},
			)
			.collect::<Vec<String>>()
			.join("\n");

		if lines.is_empty() {
			self.get_cursor_line()
		} else {
			lines
		}
	}

	pub fn down(&mut self, steps: usize) {
		if let Some(i) = self.cursor_position() {
			self.cursor_move(i as isize + steps as isize);
		}
	}

	pub fn up(&mut self, steps: usize) {
		if let Some(i) = self.cursor_position() {
			self.cursor_move(i as isize - steps as isize);
		}
	}

	pub fn first(&mut self) {
		self.cursor_move(FIRST_INDEX as isize);
	}

	pub fn last(&mut self) {
		self.cursor_move(self.last_index() as isize);
	}

	pub fn select(&mut self) {
		if let Some(i) = self.cursor_position() {
			self.selected[i] = true;
		}
	}

	pub fn unselect(&mut self) {
		if let Some(i) = self.cursor_position() {
			self.selected[i] = false;
		}
	}

	pub fn select_toggle(&mut self) {
		if let Some(i) = self.cursor_position() {
			self.selected[i] = !self.selected[i];
		}
	}

	// TODO: optimize
	pub fn select_all(&mut self) {
		self.selected = vec![true; self.lines.len()];
	}

	pub fn unselect_all(&mut self) {
		self.selected = vec![false; self.lines.len()];
	}

	fn last_index(&self) -> usize {
		if self.lines.is_empty() {
			0
		} else {
			self.lines.len() - 1
		}
	}
}
