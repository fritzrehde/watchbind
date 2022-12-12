use std::cmp::min;
// use tui::widgets::ListState;
use tui::{
	widgets::{Table, TableState, Row, Cell},
  backend::Backend,
	Frame,
};
use crate::style::Styles;

// TODO: remove lines by read only access to nth item of iterator
// TODO: learn why lifetime is needed here
pub struct StatefulList<'a> {
	pub table: Table<'a>,
	// pub lines: Vec<String>,
	pub state: TableState,
	// pub state: ListState,
	lines: Vec<String>,
	styles: Styles,
}

const FIRST_INDEX: usize = 0;

// TODO: just apply styles once in new()
impl StatefulList<'_> {
	pub fn new(lines: Vec<String>, styles: &Styles) -> StatefulList {
		StatefulList {
			lines: lines.clone(),
			table: Self::create_table(lines, styles),
			state: TableState::default(),
			styles: *styles,
			// state: ListState::default(),
		}
	}

	pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
		// TODO: maybe remove clone to make more efficient
		frame.render_stateful_widget(self.table.clone(), frame.size(), &mut self.state);
	}

	fn create_table(lines: Vec<String>, styles: &Styles) -> Table {
		// let rows: Vec<Row> = lines.iter().map(|line| Row::new(vec![Cell::from(" "), Cell::from(*line)])).collect();
		let rows: Vec<Row> = lines.iter().map(|line| Row::new(vec![Cell::from(" "), Cell::from(line.as_ref())])).collect();
		Table::new(rows)
			.style(styles.style)
			.highlight_style(styles.highlight_style)

		// let lines: Vec<ListItem> = state
		// 	.lines
		// 	.iter()
		// 	.map(|i| ListItem::new(i.as_ref()))
		// 	.collect();
		// // let lines = vec![
		// // 	ListItem::new("line one"),
		// // 	ListItem::new(""),
		// // 	ListItem::new("line four"),
		// // ];
		// let list = List::new(lines)
		// 	.style(styles.style)
		// 	.highlight_style(styles.highlight_style);

	}

	pub fn set_lines(&mut self, lines: Vec<String>) {
		self.lines = lines.clone();
		self.table = Self::create_table(lines, &self.styles);
		self.calibrate_selected_line(); // TODO: optimize through earlier if statements
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
