use crate::command::{Command, ExecutableCommand};
use crate::ui::{RequestedAction, State};
use anyhow::Result;
use parse_display::{Display, FromStr};

#[derive(Clone, FromStr, Display, PartialEq, PartialOrd, Eq, Ord)]
#[display(style = "kebab-case")]
pub enum Operation {
    Exit,
    Reload,
    HelpShow,
    HelpHide,
    HelpToggle,

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
    pub async fn execute(&self, state: &mut State) -> Result<RequestedAction> {
        match self {
            Self::MoveCursor(MoveCursor::Down(steps)) => state.move_down(*steps),
            Self::MoveCursor(MoveCursor::Up(steps)) => state.move_up(*steps),
            Self::MoveCursor(MoveCursor::First) => state.move_to_first(),
            Self::MoveCursor(MoveCursor::Last) => state.move_to_last(),
            Self::SelectLine(SelectOperation::Select) => state.select(),
            Self::SelectLine(SelectOperation::Unselect) => state.unselect(),
            Self::SelectLine(SelectOperation::ToggleSelection) => state.toggle_selection(),
            Self::SelectLine(SelectOperation::SelectAll) => state.select_all(),
            Self::SelectLine(SelectOperation::UnselectAll) => state.unselect_all(),
            Self::HelpShow => state.show_help_menu(),
            Self::HelpHide => state.hide_help_menu(),
            Self::HelpToggle => state.toggle_help_menu(),
            Self::Reload => return Ok(RequestedAction::ReloadCommand),
            Self::Exit => return Ok(RequestedAction::Exit),
            Self::Execute(command) => {
                // TODO: get_selected_lines is sync and computationally intensive, maybe use spawn_blocking

                let mut executable_command =
                    ExecutableCommand::new(command, state.get_selected_lines());

                if command.is_blocking() {
                    return Ok(RequestedAction::ExecuteBlockingSubcommand(
                        executable_command,
                    ));
                }
                executable_command.execute().await?;
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
            "cursor down 42".parse(),
            Ok(Operation::MoveCursor(MoveCursor::Down(42)))
        ));
        assert!(matches!(
            "cursor up 24".parse(),
            Ok(Operation::MoveCursor(MoveCursor::Up(24)))
        ));
    }

    #[test]
    fn test_parse_move_cursor_invalid_step_size() {
        assert!("cursor down -42".parse::<Operation>().is_err());
        assert!("cursor up -24".parse::<Operation>().is_err());
    }
}
