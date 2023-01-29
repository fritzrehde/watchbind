use crate::command::Command;
use crate::ui::{RequestedAction, State};
use anyhow::{Context, Result};
use std::str::FromStr;

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
	pub fn execute(&self, state: &mut State) -> Result<RequestedAction> {
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
					return Ok(RequestedAction::Unblock);
				}
			}
		};
		Ok(RequestedAction::Continue)
	}
}

impl FromStr for Operation {
	type Err = anyhow::Error;
	fn from_str(src: &str) -> Result<Self, Self::Err> {
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
			_ => Self::Execute(Command::new(src.to_owned())),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_move_cursor() {
		assert!(matches!(
			"down 42".parse(),
			Ok(Operation::MoveCursor(MoveCursor::Down(42)))
		));
		assert!(matches!(
			"up 24".parse(),
			Ok(Operation::MoveCursor(MoveCursor::Up(24)))
		));
	}

	#[test]
	fn test_parse_move_cursor_invalid_step_size() {
		assert!("down -42".parse::<Operation>().is_err());
		assert!("up -24".parse::<Operation>().is_err());
	}
}
