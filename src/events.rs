use tui::widgets::ListState;

pub struct Events {
	pub items: Vec<String>,
	pub state: ListState,
}

impl Events {
	pub fn new(items: Vec<String>) -> Events {
		Events {
			items,
			state: ListState::default(),
		}
	}

	pub fn set_items(&mut self, items: Vec<String>) {
		self.items = items;
		self.calibrate();
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
			},
			None => None,
		};
		self.state.select(i);
	}

	pub fn next(&mut self) {
		let last = self.last_index();
		let i = match self.state.selected() {
			Some(i) => {
				if i >= last {
					last
				} else {
					i + 1
				}
			}
			None => 0,
		};
		self.state.select(Some(i));
	}

	pub fn previous(&mut self) {
		let first = self.first_index();
		let i = match self.state.selected() {
			Some(i) => {
				if i == first {
					first
				} else {
					i - 1
				}
			}
			None => self.last_index(),
		};
		self.state.select(Some(i));
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
		self.items.len() - 1
	}
}
