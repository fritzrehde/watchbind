mod lines;

use crate::config::Styles;
use anyhow::Result;
use itertools::izip;
use lines::Lines;
use tui::{
	backend::Backend,
	layout::Constraint,
	widgets::{Row, Table, TableState},
	Frame,
};

const FIRST_INDEX: usize = 0;

pub struct State {
	lines: Lines,
	selected: Vec<bool>,
	styles: Styles,
	cursor: Option<usize>,
	// TODO: deprecate in future
	table_state: TableState,
}

impl State {
	pub fn new(field_separator: Option<String>, styles: Styles) -> Self {
		Self {
			lines: Lines::new(field_separator, styles.line),
			selected: vec![],
			styles,
			cursor: None,
			table_state: TableState::default(),
		}
	}

	pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
		// TODO: do as much as possible in update_lines to improve performance
		// TODO: replace with self.lines.iter() again
		let rows: Vec<Row> = izip!(&self.lines, self.selected.iter())
			.map(|(line, &selected)| {
				let style = if selected {
					self.styles.selected
				} else {
					self.styles.line
				};
				line.draw(style)
			})
			.collect();

		let table = Table::new(rows)
			.widths(&[Constraint::Length(1), Constraint::Percentage(100)])
			.column_spacing(0);

		frame.render_stateful_widget(table, frame.size(), &mut self.table_state);
	}

	pub fn update_lines(&mut self, lines: String) -> Result<()> {
		self.lines.update(lines)?;
		self.selected.resize(self.lines.len(), false);
		self.cursor_calibrate();
		Ok(())
	}

	fn cursor_position(&mut self) -> Option<usize> {
		self.cursor
	}

	fn cursor_move(&mut self, index: isize) {
		let old = self.cursor_position();
		let new = if self.lines.is_empty() {
			None
		} else {
			let first = FIRST_INDEX as isize;
			let last = self.last_index() as isize;
			Some(index.clamp(first, last) as usize)
		};

		self.cursor = new;
		self.table_state.select(self.cursor);
		self.cursor_adjust_style(old, new);
	}

	fn cursor_calibrate(&mut self) {
		match self.cursor_position() {
			None => self.first(),
			Some(i) => self.cursor_move(i as isize),
		};
	}

	fn cursor_adjust_style(&mut self, old: Option<usize>, new: Option<usize>) {
		if let Some(old_index) = old {
			self.lines.update_style(old_index, self.styles.line);
		}
		if let Some(new_index) = new {
			self.lines.update_style(new_index, self.styles.cursor);
		}
	}

	fn get_cursor_line(&mut self) -> Option<String> {
		if let Some(i) = self.cursor_position() {
			self.lines.get_unformatted(i)
		} else {
			None
		}
	}

	pub fn get_selected_lines(&mut self) -> Option<String> {
		if self.selected.contains(&true) {
			let lines: String = izip!(self.lines.unformatted(), self.selected.iter())
				.filter_map(|(line, &selected)| selected.then(|| line.to_owned()))
				.collect::<Vec<String>>()
				.join("\n");
			Some(lines)
		} else {
			self.get_cursor_line()
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

	pub fn select_all(&mut self) {
		self.selected.fill(true);
	}

	pub fn unselect_all(&mut self) {
		self.selected.fill(false);
	}

	fn last_index(&self) -> usize {
		if self.lines.is_empty() {
			0
		} else {
			self.lines.len() - 1
		}
	}
}
