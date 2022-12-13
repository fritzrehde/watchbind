use std::cmp::min;
use crate::style::Styles;
use tui::{
	backend::Backend,
	widgets::{Cell, Row, Table, TableState},
	layout::Constraint,
	Frame,
};

const FIRST_INDEX: usize = 0;

pub struct StatefulList {
	lines: Vec<String>,
	state: TableState,
	styles: Styles,
}

impl StatefulList {
	pub fn new(lines: Vec<String>, styles: &Styles) -> StatefulList {
		StatefulList {
			lines,
			state: TableState::default(),
			styles: *styles,
		}
	}

	pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
		// TODO: remove clone()
		let rows: Vec<Row> = self.lines
			.iter()
			// .map(|line| Row::new(vec![Cell::from(" "), Cell::from(line.clone())]))
			.map(|line| Row::new(vec![Cell::from(line.clone())]))
			.collect();

		// TODO: still very hacky
		let table = Table::new(rows)
			.style(self.styles.style)
			.highlight_style(self.styles.highlight_style)
			.widths(&[Constraint::Percentage(100)])
			.column_spacing(0);

		frame.render_stateful_widget(table, frame.size(), &mut self.state);
	}

	pub fn set_lines(&mut self, lines: Vec<String>) {
		self.lines = lines.clone();
		// TODO: optimize through earlier if statements
		self.calibrate_selected_line();
	}

	pub fn get_selected_line(&mut self) -> &str {
		if let Some(i) = self.state.selected() {
			if let Some(line) = self.lines.get(i) {
				return line;
			}
		}
		// no line selected => LINE=""
		""
	}

	// if selected line no longer exists, select last line
	pub fn calibrate_selected_line(&mut self) {
		let last = self.last_index();
		let i = match self.state.selected() {
			Some(i) => Some(min(i, last)),
			None => None,
		};
		self.state.select(i);
	}

	pub fn down(&mut self, steps: usize) {
		if steps != 0 {
			let new_i = match self.state.selected() {
				Some(i) => i + steps,
				None => FIRST_INDEX + steps - 1,
			};
			// check if in bounds
			let i = min(new_i, self.last_index());
			self.state.select(Some(i));
		}
	}

	pub fn up(&mut self, steps: usize) {
		if steps != 0 {
			let new_i = match self.state.selected() {
				Some(i) => i,
				None => self.last_index() + 1,
			};
			// check if in bounds
			let i = new_i.checked_sub(steps).unwrap_or(FIRST_INDEX);
			self.state.select(Some(i));
		}
	}

	pub fn unselect(&mut self) {
		self.state.select(None);
	}

	pub fn first(&mut self) {
		self.state.select(Some(self.first_index()));
	}

	pub fn last(&mut self) {
		self.state.select(Some(self.last_index()));
	}

	fn first_index(&self) -> usize {
		0
	}

	fn last_index(&self) -> usize {
		if self.lines.is_empty() {
			0
		} else {
			self.lines.len() - 1
		}
	}
}
