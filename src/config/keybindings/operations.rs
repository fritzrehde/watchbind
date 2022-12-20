use crate::exec::{exec_blocking, exec_non_blocking};
use crate::ui::{State, Event, RequestedAction};
use anyhow::{Context, Result};
use std::{
	collections::VecDeque,
	str::FromStr,
	sync::mpsc::Sender,
};

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

	pub fn from_vec(ops: Vec<String>) -> Result<Self> {
		let operations = ops
			.into_iter()
			.map(|op| Ok(op.parse()?))
			.collect::<Result<_>>()?;
		Ok(Self { operations })
	}
}

#[derive(Clone)]
pub enum Operation {
	Exit,
	Reload,
	MoveCursor(MoveCursor),
	SelectLine(SelectOperation),
	Execute(Command),
}

// TODO: add support for goto nth line
#[derive(Clone)]
pub enum MoveCursor {
	Down(usize),
	Up(usize),
	First,
	Last,
}

#[derive(Clone)]
pub enum SelectOperation {
	Select,
	Unselect,
	Toggle,
	SelectAll,
	UnselectAll,
}

#[derive(Clone)]
pub struct Command {
	pub command: String,
	pub blocking: bool,
}

impl Operation {
	pub fn execute(
		self,
		state: &mut State,
		event_tx: &Sender<Event>,
	) -> Result<RequestedAction> {
		match self {
			Self::MoveCursor(MoveCursor::Down(steps)) => state.down(steps),
			Self::MoveCursor(MoveCursor::Up(steps)) => state.up(steps),
			Self::MoveCursor(MoveCursor::First) => state.first(),
			Self::MoveCursor(MoveCursor::Last) => state.last(),
			Self::SelectLine(SelectOperation::Select) => state.select(),
			Self::SelectLine(SelectOperation::Unselect) => state.unselect(),
			Self::SelectLine(SelectOperation::Toggle) => state.select_toggle(),
			Self::SelectLine(SelectOperation::SelectAll) => state.select_all(),
			Self::SelectLine(SelectOperation::UnselectAll) => state.unselect_all(),
			Self::Reload => return Ok(RequestedAction::Reload),
			Self::Exit => return Ok(RequestedAction::Exit),
			Self::Execute(command) => {
				let lines = state.get_selected_lines();
				return Ok(if command.blocking {
					exec_blocking(&command.command, &lines, event_tx.clone());
					RequestedAction::Block
				} else {
					exec_non_blocking(&command.command, &lines)?;
					RequestedAction::Continue
				});
			}
		};
		Ok(RequestedAction::Continue)
	}
}

impl FromStr for Operation {
	type Err = anyhow::Error;
	fn from_str(src: &str) -> Result<Self> {
		// TODO: consider creating type "StepSize"
		let parse_steps = |steps: &str, src| {
			steps
				.parse()
				.with_context(|| format!("Invalid step size \"{steps}\" provided in keybinding: \"{src}\""))
		};
		Ok(match src.split_whitespace().collect::<Vec<&str>>()[..] {
			["exit"] => Self::Exit,
			["reload"] => Self::Reload,
			["down"] => Self::MoveCursor(MoveCursor::Down(1)),
			["up"] => Self::MoveCursor(MoveCursor::Up(1)),
			["down", steps] => Self::MoveCursor(MoveCursor::Down(parse_steps(steps, src)?)),
			["up", steps] => Self::MoveCursor(MoveCursor::Up(parse_steps(steps, src)?)),
			["first"] => Self::MoveCursor(MoveCursor::First),
			["last"] => Self::MoveCursor(MoveCursor::Last),
			["select"] => Self::SelectLine(SelectOperation::Select),
			["unselect"] => Self::SelectLine(SelectOperation::Unselect),
			["select-toggle"] => Self::SelectLine(SelectOperation::Toggle),
			["select-all"] => Self::SelectLine(SelectOperation::SelectAll),
			["unselect-all"] => Self::SelectLine(SelectOperation::UnselectAll),
			_ => Self::Execute(Command {
				command: src.to_string(),
				blocking: !src.contains("&"),
			}),
		})
	}
}
