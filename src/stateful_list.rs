use crate::style::Styles;
use itertools::izip;
use std::cmp::min;
use tui::{
	backend::Backend,
	layout::Constraint,
	style::{Color, Style},
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

	pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
		let rows: Vec<Row> = izip!(self.lines.iter(), self.selected.iter())
			.map(|(line, &selected)| {
				Row::new(vec![
					Cell::from(" ").style(if selected {
						// TODO: allow customize color magenta
						Style::reset().bg(Color::Magenta)
					} else {
						Style::reset()
					}),
					// TODO: remove clone()
					Cell::from(line.clone()),
				])
			})
			.collect();

		let table = Table::new(rows)
			.style(self.styles.style)
			.highlight_style(self.styles.highlight_style)
			// .widths(&[Constraint::Percentage(100)])
			.widths(&[Constraint::Length(1), Constraint::Percentage(100)])
			.column_spacing(1);

		frame.render_stateful_widget(table, frame.size(), &mut self.state);
	}

	pub fn set_lines(&mut self, lines: Vec<String>) {
		self.selected.resize(lines.len(), false);
		self.lines = lines;
		// TODO: optimize through earlier if statements
		self.calibrate_selected_line();
	}

	fn cursor_position(&mut self) -> Option<usize> {
		self.state.selected()
	}

	fn cursor_move(&mut self, index: usize) {
		self.state.select(Some(index));
	}

	pub fn get_selected_line(&mut self) -> &str {
		if let Some(i) = self.cursor_position() {
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
		let i = match self.cursor_position() {
			Some(i) => Some(min(i, last)),
			None => None,
		};
		self.state.select(i);
	}

	// TODO: implement using cursor
	pub fn down(&mut self, steps: usize) {
		// self.cursor_move(self.cursor_position() + steps);
		if steps != 0 {
			let new_i = match self.cursor_position() {
				Some(i) => i + steps,
				None => FIRST_INDEX + steps - 1,
			};
			// check if in bounds
			let i = min(new_i, self.last_index());
			self.cursor_move(i);
		}
	}

	pub fn up(&mut self, steps: usize) {
		if steps != 0 {
			let new_i = match self.cursor_position() {
				Some(i) => i,
				None => self.last_index() + 1,
			};
			// check if in bounds
			let i = new_i.checked_sub(steps).unwrap_or(FIRST_INDEX);
			self.state.select(Some(i));
		}
	}

	pub fn select(&mut self) {
		if let Some(i) = self.cursor_position() {
			self.selected[i] = true;
		}
	}

	pub fn select_toggle(&mut self) {
		if let Some(i) = self.cursor_position() {
			self.selected[i] = !self.selected[i];
		}
	}

	pub fn unselect(&mut self) {
		self.selected = vec![false; self.lines.len()];
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
