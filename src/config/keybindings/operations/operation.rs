use crate::command::Command;
use crate::ui::{RequestedAction, State};
use anyhow::Result;
use parse_display::{Display, FromStr};

#[derive(Clone, FromStr, Display, PartialEq, PartialOrd, Eq, Ord)]
#[display(style = "kebab-case")]
pub enum Operation {
    Exit,
    Reload,
    Help,

    #[display("cursor {0}")]
    MoveCursor(MoveCursor),

    #[display("{0}")]
    SelectLine(SelectOperation),

    #[display("exec -- {0}")]
    Execute(Command),
}

// TODO: add support for goto nth line
#[derive(Clone, FromStr, Display, PartialEq, PartialOrd, Eq, Ord)]
#[display(style = "lowercase")]
pub enum MoveCursor {
    #[display("down {0}")]
    Down(usize),

    #[display("up {0}")]
    Up(usize),

    First,
    Last,
}

#[derive(Clone, FromStr, Display, PartialEq, PartialOrd, Eq, Ord)]
#[display(style = "kebab-case")]
pub enum SelectOperation {
    Select,
    Unselect,
    ToggleSelection,
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
            Self::SelectLine(SelectOperation::ToggleSelection) => state.select_toggle(),
            Self::SelectLine(SelectOperation::SelectAll) => state.select_all(),
            Self::SelectLine(SelectOperation::UnselectAll) => state.unselect_all(),
            Self::Help => state.toggle_help_menu(),
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
