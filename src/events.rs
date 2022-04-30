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
		self.state = ListState::default();
	}

	pub fn next(&mut self) {
		let last = self.get_last();
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
		let first = self.get_first();
		let i = match self.state.selected() {
			Some(i) => {
				if i == first {
					first
				} else {
					i - 1
				}
			}
			None => 0,
		};
		self.state.select(Some(i));
	}

	pub fn unselect(&mut self) {
		self.state.select(None);
	}

	pub fn first(&mut self) {
		self.state.select(Some(self.get_first()));
	}

	pub fn last(&mut self) {
		self.state.select(Some(self.get_last()));
	}

	fn get_first(&self) -> usize {
		0
	}

	fn get_last(&self) -> usize {
		self.items.len() - 1
	}
}
