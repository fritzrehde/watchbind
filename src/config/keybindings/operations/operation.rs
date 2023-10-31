use crate::command::{Blocking, CommandBuilder, NonBlocking, WithEnv, WithOutput};
use crate::ui::{EnvVariable, EnvVariables, Event, RequestedAction, State};
use anyhow::Result;
use parse_display::{Display, FromStr};
use std::str;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

// TODO: use some rust pattern (with types) instead of hardcoded Operation{,Parsed} variants

/// The version of Operation used for parsing and displaying. The reason we
/// can't parse directly into Operation is because any operations that execute
/// something need to receive access to the globally set environment variables.
#[derive(FromStr, Display, PartialEq, PartialOrd, Eq, Ord)]
#[display(style = "kebab-case")]
pub enum OperationParsed {
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
    ExecuteBlocking(String),

    #[display("exec & -- {0}")]
    ExecuteNonBlocking(String),

    #[display("set-env {0} -- {1}")]
    SetEnv(EnvVariable, String),

    #[display("unset-env {0}")]
    UnsetEnv(EnvVariable),

    #[display("read-into-env {0}")]
    ReadIntoEnv(EnvVariable),
}

pub enum Operation {
    Exit,
    Reload,
    HelpShow,
    HelpHide,
    HelpToggle,
    MoveCursor(MoveCursor),
    SelectLine(SelectOperation),
    ExecuteBlocking(Arc<CommandBuilder<Blocking, WithEnv>>),
    ExecuteNonBlocking(Arc<CommandBuilder<NonBlocking, WithEnv>>),
    SetEnv(
        EnvVariable,
        Arc<CommandBuilder<Blocking, WithEnv, WithOutput>>,
    ),

    UnsetEnv(EnvVariable),
    ReadIntoEnv(EnvVariable),
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
    pub async fn execute(
        &self,
        state: &mut State,
        event_tx: &Sender<Event>,
    ) -> Result<RequestedAction> {
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
            Self::HelpShow => state.show_help_menu().await,
            Self::HelpHide => state.hide_help_menu(),
            Self::HelpToggle => state.toggle_help_menu().await,
            Self::Reload => return Ok(RequestedAction::ReloadWatchedCommand),
            Self::Exit => return Ok(RequestedAction::Exit),
            Self::ExecuteNonBlocking(non_blocking_cmd) => {
                state.add_lines_to_env().await?;
                non_blocking_cmd.execute().await?;
            }
            Self::ExecuteBlocking(blocking_cmd) => {
                state.add_lines_to_env().await?;

                // TODO: these clones are preventable by using Arc<> (I think Arc<Mutex> isn't required because executing them doesn't mutate them)
                let blocking_cmd = blocking_cmd.clone();
                let event_tx = event_tx.clone();
                tokio::spawn(async move {
                    let result = blocking_cmd.execute().await;

                    // Ignore whether the sender has closed channel.
                    let _ = event_tx.send(Event::SubcommandCompleted(result)).await;
                });

                return Ok(RequestedAction::ExecutingBlockingSubcommand);
            }
            Self::SetEnv(env_variable, blocking_cmd) => {
                state.add_lines_to_env().await?;

                // TODO: these clones are preventable by using Arc<> (I think Arc<Mutex> isn't required because executing them doesn't mutate them)
                let blocking_cmd = blocking_cmd.clone();
                let env_variable = env_variable.to_owned();
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
            Self::UnsetEnv(env) => state.unset_env_var(env).await,
            Self::ReadIntoEnv(env) => {
                state.request_read_into_env(env).await;
                return Ok(RequestedAction::UserInputTextfield);
            }
        };
        Ok(RequestedAction::Continue)
    }

    /// Convert the parsed form into the normal, runtime Operation form.
    pub fn from_parsed(parsed: OperationParsed, env_variables: &Arc<Mutex<EnvVariables>>) -> Self {
        match parsed {
            OperationParsed::Exit => Self::Exit,
            OperationParsed::Reload => Self::Reload,
            OperationParsed::HelpShow => Self::HelpShow,
            OperationParsed::HelpHide => Self::HelpHide,
            OperationParsed::HelpToggle => Self::HelpToggle,
            OperationParsed::MoveCursor(x) => Self::MoveCursor(x),
            OperationParsed::SelectLine(x) => Self::SelectLine(x),
            OperationParsed::ExecuteBlocking(cmd) => Self::ExecuteBlocking(Arc::new(
                CommandBuilder::new(cmd)
                    .blocking()
                    .with_env(env_variables.clone()),
            )),
            OperationParsed::ExecuteNonBlocking(cmd) => Self::ExecuteNonBlocking(Arc::new(
                CommandBuilder::new(cmd).with_env(env_variables.clone()),
            )),
            OperationParsed::SetEnv(env_var, cmd) => Self::SetEnv(
                env_var,
                Arc::new(
                    CommandBuilder::new(cmd)
                        .blocking()
                        .with_output()
                        .with_env(env_variables.clone()),
                ),
            ),
            OperationParsed::UnsetEnv(x) => Self::UnsetEnv(x),
            OperationParsed::ReadIntoEnv(x) => Self::ReadIntoEnv(x),
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
            Ok(OperationParsed::MoveCursor(MoveCursor::Down(42)))
        ));
        assert!(matches!(
            "cursor up 24".parse(),
            Ok(OperationParsed::MoveCursor(MoveCursor::Up(24)))
        ));
    }

    #[test]
    fn test_parse_move_cursor_invalid_step_size() {
        assert!("cursor down -42".parse::<OperationParsed>().is_err());
        assert!("cursor up -24".parse::<OperationParsed>().is_err());
    }
}
