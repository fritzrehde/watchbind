use anyhow::{Context, Result};
use parse_display::{Display, FromStr};
use std::str;
use std::sync::Arc;
use strum::{EnumIter, EnumMessage};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;

use crate::config::KeyEvent;
use crate::ui::{EnvVariable, EnvVariables, Event, RequestedAction, State};
use crate::utils::command::{
    Blocking, CommandBuilder, InheritedIO, NonBlocking, NonInterruptible, WithEnv, WithOutput,
};

#[derive(Display)]
#[display("{parsed}")]
pub struct Operation {
    /// Used for executing an operation.
    pub executable: OperationExecutable,

    /// Used for displaying an operation with `fmt::Display`.
    parsed: OperationParsed,
}

// TODO: use some rust pattern (with types) instead of hardcoded OperationParsed variant

/// The version of Operation used for parsing and displaying. The reason we
/// can't parse directly into Operation is because any operations that execute
/// something need to receive access to the globally set environment variables.
#[derive(
    Debug,
    // For using as key in hashmap
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Clone,
    // For displaying and parsing
    Display,
    FromStr,
    // For displaying all possible variants
    EnumIter,
    EnumMessage,
)]
#[display(style = "kebab-case")]
pub enum OperationParsed {
    Exit,
    Reload,

    #[display("cursor up {0}")]
    #[strum(message = "cursor up <N>")]
    MoveCursorUp(usize),

    #[display("cursor down {0}")]
    #[strum(message = "cursor down <N>")]
    MoveCursorDown(usize),

    #[display("cursor first")]
    MoveCursorFirst,

    #[display("cursor last")]
    MoveCursorLast,

    #[display("select")]
    SelectLine,

    #[display("unselect")]
    UnselectLine,

    #[display("toggle-selection")]
    ToggleLineSelection,

    #[display("select-all")]
    SelectAllLines,

    #[display("unselect-all")]
    UnselectAllLines,

    #[display("exec -- {0}")]
    #[strum(message = "exec -- <CMD>")]
    ExecuteBlocking(String),

    #[display("exec & -- {0}")]
    #[strum(message = "exec & -- <CMD>")]
    ExecuteNonBlocking(String),

    #[display("exec tui -- {0}")]
    #[strum(message = "exec tui & -- <TUI-CMD>")]
    ExecuteTUI(String),

    #[display("set-env {0} -- {1}")]
    #[strum(message = "set-env <ENV> -- <CMD>")]
    SetEnv(EnvVariable, String),

    #[display("unset-env {0}")]
    #[strum(message = "unset-env <ENV>")]
    UnsetEnv(EnvVariable),

    #[display("read-into-env {0}")]
    #[strum(message = "read-into-env <ENV>")]
    ReadIntoEnv(EnvVariable),

    HelpShow,
    HelpHide,
    HelpToggle,
}

pub enum OperationExecutable {
    Exit,
    Reload,
    HelpShow,
    HelpHide,
    HelpToggle,
    MoveCursor(MoveCursor),
    SelectLine(SelectOperation),
    // TODO: document why we have an Arc (probably because it's shared across threads, but why? is it even necessary to share across threads given async)
    ExecuteBlocking(Arc<CommandBuilder<Blocking, WithEnv>>),
    ExecuteNonBlocking(Arc<CommandBuilder<NonBlocking, WithEnv>>),
    ExecuteTUI(Arc<CommandBuilder<Blocking, WithEnv, InheritedIO, NonInterruptible>>),
    SetEnv(
        EnvVariable,
        Arc<CommandBuilder<Blocking, WithEnv, WithOutput>>,
    ),
    UnsetEnv(EnvVariable),
    ReadIntoEnv(EnvVariable),
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum MoveCursor {
    Down(usize),
    Up(usize),
    First,
    Last,
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum SelectOperation {
    Select,
    Unselect,
    ToggleSelection,
    SelectAll,
    UnselectAll,
}

impl Operation {
    /// Execute the operation given the current `State` of the program. Perform
    /// any additional async communication with the main event loop through the
    /// `event_tx` channel. Also use the `key_event` that triggered this
    /// operation for printing helpful error messages.
    pub async fn execute(
        &self,
        state: &mut State,
        event_tx: &Sender<Event>,
        key_event: &KeyEvent,
    ) -> Result<RequestedAction> {
        match &self.executable {
            OperationExecutable::MoveCursor(MoveCursor::Down(steps)) => state.move_down(*steps),
            OperationExecutable::MoveCursor(MoveCursor::Up(steps)) => state.move_up(*steps),
            OperationExecutable::MoveCursor(MoveCursor::First) => state.move_to_first(),
            OperationExecutable::MoveCursor(MoveCursor::Last) => state.move_to_last(),
            OperationExecutable::SelectLine(SelectOperation::Select) => state.select(),
            OperationExecutable::SelectLine(SelectOperation::Unselect) => state.unselect(),
            OperationExecutable::SelectLine(SelectOperation::ToggleSelection) => {
                state.toggle_selection()
            }
            OperationExecutable::SelectLine(SelectOperation::SelectAll) => state.select_all(),
            OperationExecutable::SelectLine(SelectOperation::UnselectAll) => state.unselect_all(),
            OperationExecutable::HelpShow => state.show_help_menu().await,
            OperationExecutable::HelpHide => state.hide_help_menu(),
            OperationExecutable::HelpToggle => state.toggle_help_menu().await,
            OperationExecutable::Reload => return Ok(RequestedAction::ReloadWatchedCommand),
            OperationExecutable::Exit => return Ok(RequestedAction::Exit),
            OperationExecutable::ExecuteNonBlocking(non_blocking_cmd) => {
                state.add_cursor_and_selected_lines_to_env().await;
                non_blocking_cmd.execute().await?;
                state.remove_cursor_and_selected_lines_from_env().await;
            }
            OperationExecutable::ExecuteBlocking(blocking_cmd) => {
                state.add_cursor_and_selected_lines_to_env().await;

                let blocking_cmd = Arc::clone(blocking_cmd);
                let event_tx = event_tx.clone();
                // TODO: inefficient: creating Strings that are only used in the (rare) error-case
                let (op_to_string, key_to_string) = (self.to_string(), key_event.to_string());
                tokio::spawn(async move {
                    let result = blocking_cmd.execute().await.with_context(|| {
                        format!("Execution of blocking subcommand \"{}\", triggered by key event \"{}\", failed", op_to_string, key_to_string)
                    });

                    // Ignore whether the sender has closed channel.
                    let _ = event_tx.send(Event::SubcommandCompleted(result)).await;
                });

                // Don't call state.remove_cursor_and_selected_lines_from_env()
                // here, because it would race with the spawned Tokio task. It
                // will be called once this subcommand completes.

                return Ok(RequestedAction::ExecutingBlockingSubcommand);
            }
            OperationExecutable::ExecuteTUI(tui_cmd) => {
                state.add_cursor_and_selected_lines_to_env().await;

                // Create channels for waiting until TUI has actually been hidden.
                let (tui_hidden_tx, mut tui_hidden_rx) = mpsc::channel(1);

                let tui_cmd = Arc::clone(tui_cmd);
                let event_tx = event_tx.clone();
                // TODO: inefficient: creating Strings that are only used in the (rare) error-case
                let (op_to_string, key_to_string) = (self.to_string(), key_event.to_string());
                tokio::spawn(async move {
                    // Wait until TUI has actually been hidden.
                    let _ = tui_hidden_rx.recv().await;

                    let result = tui_cmd.execute().await.with_context(|| {
                        format!("Execution of TUI subcommand \"{}\", triggered by key event \"{}\", failed", op_to_string, key_to_string)
                    });

                    // Ignore whether the sender has closed channel.
                    let _ = event_tx.send(Event::TUISubcommandCompleted(result)).await;
                });

                // Don't call state.remove_cursor_and_selected_lines_from_env()
                // here, because it would race with the spawned Tokio task. It
                // will be called once this subcommand completes.

                return Ok(RequestedAction::ExecutingTUISubcommand(tui_hidden_tx));
            }
            OperationExecutable::SetEnv(env_variable, blocking_cmd) => {
                state.add_cursor_and_selected_lines_to_env().await;

                let blocking_cmd = blocking_cmd.clone();
                let env_variable = env_variable.clone();
                let event_tx = event_tx.clone();
                tokio::spawn(async move {
                    let result = blocking_cmd.execute().await.map(|output| {
                        [(env_variable, output)]
                            .into_iter()
                            .collect::<EnvVariables>()
                    });

                    // Ignore whether the sender has closed channel.
                    let _ = event_tx
                        .send(Event::SubcommandForEnvCompleted(result))
                        .await;
                });

                return Ok(RequestedAction::ExecutingBlockingSubcommandForEnv);
            }
            OperationExecutable::UnsetEnv(env) => state.unset_env(env).await,
            OperationExecutable::ReadIntoEnv(env) => state.read_into_env(env).await,
        };
        Ok(RequestedAction::Continue)
    }

    /// Convert the parsed form into the normal, runtime executable form. The
    /// `env_variables` is required so it can be passed to the `SetEnv` command.
    pub fn from_parsed(parsed: OperationParsed, env_variables: &Arc<Mutex<EnvVariables>>) -> Self {
        let operation_executable = match parsed.clone() {
            OperationParsed::Exit => OperationExecutable::Exit,
            OperationParsed::Reload => OperationExecutable::Reload,
            OperationParsed::MoveCursorUp(n) => OperationExecutable::MoveCursor(MoveCursor::Up(n)),
            OperationParsed::MoveCursorDown(n) => {
                OperationExecutable::MoveCursor(MoveCursor::Down(n))
            }
            OperationParsed::MoveCursorFirst => OperationExecutable::MoveCursor(MoveCursor::First),
            OperationParsed::MoveCursorLast => OperationExecutable::MoveCursor(MoveCursor::Last),
            OperationParsed::SelectLine => OperationExecutable::SelectLine(SelectOperation::Select),
            OperationParsed::UnselectLine => {
                OperationExecutable::SelectLine(SelectOperation::Unselect)
            }
            OperationParsed::ToggleLineSelection => {
                OperationExecutable::SelectLine(SelectOperation::ToggleSelection)
            }
            OperationParsed::SelectAllLines => {
                OperationExecutable::SelectLine(SelectOperation::SelectAll)
            }
            OperationParsed::UnselectAllLines => {
                OperationExecutable::SelectLine(SelectOperation::UnselectAll)
            }
            OperationParsed::ExecuteBlocking(cmd) => {
                OperationExecutable::ExecuteBlocking(Arc::new(
                    CommandBuilder::new(cmd)
                        .blocking()
                        .with_env(env_variables.clone()),
                ))
            }
            OperationParsed::ExecuteNonBlocking(cmd) => OperationExecutable::ExecuteNonBlocking(
                Arc::new(CommandBuilder::new(cmd).with_env(env_variables.clone())),
            ),
            OperationParsed::ExecuteTUI(cmd) => OperationExecutable::ExecuteTUI(Arc::new(
                CommandBuilder::new(cmd)
                    .blocking()
                    .inherited_io()
                    .with_env(env_variables.clone()),
            )),
            OperationParsed::SetEnv(env_var, cmd) => OperationExecutable::SetEnv(
                env_var,
                Arc::new(
                    CommandBuilder::new(cmd)
                        .blocking()
                        .with_output()
                        .with_env(env_variables.clone()),
                ),
            ),
            OperationParsed::UnsetEnv(x) => OperationExecutable::UnsetEnv(x),
            OperationParsed::ReadIntoEnv(x) => OperationExecutable::ReadIntoEnv(x),
            OperationParsed::HelpShow => OperationExecutable::HelpShow,
            OperationParsed::HelpHide => OperationExecutable::HelpHide,
            OperationParsed::HelpToggle => OperationExecutable::HelpToggle,
        };
        Self {
            executable: operation_executable,
            parsed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_move_cursor() {
        assert!(matches!(
            "cursor down 42".parse(),
            Ok(OperationParsed::MoveCursorDown(42))
        ));
        assert!(matches!(
            "cursor up 24".parse(),
            Ok(OperationParsed::MoveCursorUp(24))
        ));
    }

    #[test]
    fn test_parse_move_cursor_invalid_step_size() {
        assert!("cursor down -42".parse::<OperationParsed>().is_err());
        assert!("cursor up -24".parse::<OperationParsed>().is_err());
    }
}
