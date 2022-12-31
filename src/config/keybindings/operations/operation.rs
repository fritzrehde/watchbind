use crate::command::Command;
use crate::ui::{State, Event, RequestedAction};
use anyhow::{Context, Result};
use std::sync::mpsc::Sender;

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

impl Operation {
	pub fn from_str(src: String, event_tx: &Sender<Event>) -> Result<Self> {
		// TODO: consider creating type "StepSize"
		let parse_steps = |steps: &str| {
			steps
				.parse()
				.with_context(|| format!("Invalid step size \"{steps}\" provided in keybinding: \"{src}\""))
		};
		Ok(match src.split_whitespace().collect::<Vec<&str>>()[..] {
			["exit"] => Self::Exit,
			["reload"] => Self::Reload,
			["down"] => Self::MoveCursor(MoveCursor::Down(1)),
			["up"] => Self::MoveCursor(MoveCursor::Up(1)),
			["down", steps] => Self::MoveCursor(MoveCursor::Down(parse_steps(steps)?)),
			["up", steps] => Self::MoveCursor(MoveCursor::Up(parse_steps(steps)?)),
			["first"] => Self::MoveCursor(MoveCursor::First),
			["last"] => Self::MoveCursor(MoveCursor::Last),
			["select"] => Self::SelectLine(SelectOperation::Select),
			["unselect"] => Self::SelectLine(SelectOperation::Unselect),
			["select-toggle"] => Self::SelectLine(SelectOperation::Toggle),
			["select-all"] => Self::SelectLine(SelectOperation::SelectAll),
			["unselect-all"] => Self::SelectLine(SelectOperation::UnselectAll),
			_ => Self::Execute({
				// TODO: remove " &" from command if present
				let blocking = !src.contains(" &");
				let mut command = Command::new(src);
				if blocking {
					command.block(event_tx.clone());
				}
				command
			}),
		})
	}

	pub fn execute(
		&self,
		state: &mut State,
	) -> Result<RequestedAction> {
		match self {
			Self::MoveCursor(MoveCursor::Down(steps)) => state.down(*steps),
			Self::MoveCursor(MoveCursor::Up(steps)) => state.up(*steps),
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
				command.execute(state.get_selected_lines())?;
				if command.is_blocking() {
					return Ok(RequestedAction::Block);
				}
			},
		};
		Ok(RequestedAction::Continue)
	}
}
