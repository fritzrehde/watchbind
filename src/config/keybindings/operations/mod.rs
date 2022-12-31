mod operation;

use crate::ui::Event;
use anyhow::Result;
use operation::Operation;
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
}
