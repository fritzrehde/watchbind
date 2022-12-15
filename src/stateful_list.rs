use crate::style::Styles;
use itertools::izip;
use tui::{
	backend::Backend,
	layout::Constraint,
	widgets::{Cell, Row, Table, TableState},
	Frame,
};

const FIRST_INDEX: usize = 0;

// TODO: replace vectors with slices
pub struct StatefulList {
	lines: Vec<String>,
	selected: Vec<bool>,
	state: TableState,
	styles: Styles,
}

impl StatefulList {
	pub fn new(lines: Vec<String>, styles: &Styles) -> StatefulList {
		let mut state = StatefulList {
			selected: vec![false; lines.len()],
			lines,
			state: TableState::default(),
			styles: *styles,
		};
		state.first();
		state
	}

	// TODO: very messy formatting
	pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
		// TODO: hacky
		let cursor_index = match self.cursor_position() {
			Some(i) => i as isize,
			None => -1,
		};

		let rows: Vec<Row> = izip!(self.lines.iter(), self.selected.iter())
			.enumerate()
			.map(|(i, (line, &selected))| {
				Row::new(vec![
					Cell::from(" ").style(if selected {
						self.styles.selected
					} else {
						// Style::reset()
						self.styles.line
					}),
					Cell::from(" ".to_owned() + &line).style(if i as isize == cursor_index {
						self.styles.cursor
					} else {
						self.styles.line
					}),
				])
			})
			.collect();

		let table = Table::new(rows)
			.widths(&[Constraint::Length(1), Constraint::Percentage(100)])
			.column_spacing(0);

		frame.render_stateful_widget(table, frame.size(), &mut self.state);
	}

	pub fn set_lines(&mut self, lines: Vec<String>) {
		self.selected.resize(lines.len(), false);
		self.lines = lines;
		self.calibrate_cursor();
	}

	fn cursor_position(&mut self) -> Option<usize> {
		self.state.selected()
	}

	fn cursor_move(&mut self, index: isize) {
		let first = FIRST_INDEX as isize;
		let last = self.last_index() as isize;
		let i = if index < first {
			first
		} else if index > last {
			last
		} else {
			index
		} as usize;
		self.state.select(Some(i));
	}

	fn get_cursor_line(&mut self) -> String {
		if let Some(i) = self.cursor_position() {
			if let Some(line) = self.lines.get(i) {
				return line.clone();
			}
		}
		// no line selected => LINE=""
		"".to_string()
	}

	// if selected line no longer exists, select last line
	fn calibrate_cursor(&mut self) {
		if let Some(i) = self.cursor_position() {
			self.cursor_move(i as isize);
		}
	}

	// pub fn get_selected_lines(&mut self) -> &str {
	pub fn get_selected_lines(&mut self) -> String {
		let lines: String = izip!(self.lines.iter(), self.selected.iter())
			.filter_map(
				|(line, &selected)| {
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
		self.state.select(Some(FIRST_INDEX));
	}

	pub fn last(&mut self) {
		self.state.select(Some(self.last_index()));
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
