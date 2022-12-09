use std::cmp::min;
use tui::widgets::ListState;

pub struct Events {
	pub items: Vec<String>,
	pub state: ListState,
}

const FIRST_INDEX: usize = 0;

impl Events {
	pub fn new(items: Vec<String>) -> Events {
		Events {
			items,
			state: ListState::default(),
		}
	}

	pub fn set_items(&mut self, items: Vec<String>) {
		self.items = items;
		self.calibrate(); // TODO: optimize through earlier if statements
	}

	pub fn get_selected_line(&mut self) -> &str {
		if let Some(i) = self.state.selected() {
			if let Some(line) = self.items.get(i) {
				return line;
			}
		}
		// no line selected => LINE=""
		""
	}

	// if selected line no longer exists, select last line
	pub fn calibrate(&mut self) {
		let last = self.last_index();
		let i = match self.state.selected() {
			Some(i) => {
				if i > last {
					Some(last)
				} else {
					Some(i)
				}
			}
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
		if self.items.is_empty() {
			0
		} else {
			self.items.len() - 1
		}
	}
}
