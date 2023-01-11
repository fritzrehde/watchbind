mod operation;

pub use operation::Operation;

use crate::ui::Event;
use anyhow::Result;
use std::{collections::VecDeque, sync::mpsc::Sender};

pub struct Operations {
	operations: VecDeque<Operation>,
}

impl Operations {
	// TODO: rename to default or empty
	pub fn new() -> Self {
		Self {
			operations: VecDeque::new(),
		}
	}

	pub fn add(&mut self, added: &Self) {
		self.operations.append(&mut added.operations.clone());
	}

	pub fn next(&mut self) -> Option<Operation> {
		self.operations.pop_front()
	}

	pub fn from_vec(ops: Vec<String>) -> Result<Self> {
		let operations = ops
			.into_iter()
			.map(|op| op.parse())
			.collect::<Result<_>>()?;
		Ok(Self { operations })
	}

	pub fn add_tx(&mut self, event_tx: &Sender<Event>) {
		for op in self.operations.iter_mut() {
			op.add_tx(event_tx);
		}
	}
}
