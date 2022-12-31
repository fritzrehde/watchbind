mod operation;

pub use operation::Operation;

use crate::ui::Event;
use anyhow::Result;
use std::{collections::VecDeque, sync::mpsc::Sender};

pub struct Operations {
	operations: VecDeque<Operation>,
}

impl Operations {
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

	pub fn from_vec(ops: Vec<String>, event_tx: &Sender<Event>) -> Result<Self> {
		let operations = ops
			.into_iter()
			.map(|op| Ok(Operation::from_str(op, event_tx)?))
			.collect::<Result<_>>()?;
		Ok(Self { operations })
	}

	pub fn add_tx(&mut self, event_tx: &Sender<Event>) {
		for op in self.operations.iter_mut() {
			if let Operation::Execute(command) = op {
				command.add_tx(event_tx.clone());
			}
			// op.add_tx(event_tx.clone());
		}
	}
}
