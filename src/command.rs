use anyhow::{bail, Error, Result};
// use core::time::Duration;
use parse_display::Display;
use std::{
    borrow::Cow,
    process::{ExitStatus, Stdio},
    str::FromStr,
};
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::Receiver;

#[derive(Clone, Display, PartialEq, PartialOrd, Eq, Ord)]
#[display("{command}")]
pub struct Command {
    command: String,
    is_blocking: bool,
}

impl FromStr for Command {
    type Err = Error;
    fn from_str(command: &str) -> Result<Self, Self::Err> {
        let mut command = command.to_owned();
        let is_blocking = !command.ends_with(" &");
        if !is_blocking {
            command.truncate(command.len() - " &".len());
        }

        Ok(Self {
            command,
            is_blocking,
        })
    }
}

/// Encodes whether a command's execution was interrupted or the stdout if it
/// ran to completion.
pub enum AsyncResult {
    Stdout(String),
    Interrupted,
}

impl Command {
    /// Returns whether the command is configured to be blocking.
    pub fn is_blocking(&self) -> bool {
        self.is_blocking
    }

    /// Executes a command with the LINES env variable optionally set.
    /// This method cannot be interrupted (e.g. reloaded), and does not
    /// return the stdout of the command.
    pub async fn execute(&self, lines: Option<String>) -> Result<()> {
        let mut child = self
            .create_shell_command(lines)
            // We only need stderr in case of an error, stdout can be ignored
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?;

        if self.is_blocking {
            let exit_status = child.wait().await?;
            child_exited_successfully(exit_status, &mut child.stderr).await?;
        }

        Ok(())
    }

    /// Executes a command asynchronously. Listens for an interrupt signal and
    /// waits for the stdout of the command concurrently (at the same time).
    pub async fn capture_output(&self, interrupt_rx: &mut Receiver<()>) -> Result<AsyncResult> {
        let mut child = self
            .create_shell_command(None)
            // Keep both stdout and stderr
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        tokio::select! {
            _ = interrupt_rx.recv() => {
                child.kill().await.unwrap();
                Ok(AsyncResult::Interrupted)
            },
            exit_status = child.wait() => {
                child_exited_successfully(exit_status?, &mut child.stderr).await?;

                // Read stdout
                let mut stdout = String::new();
                child.stdout.unwrap().read_to_string(&mut stdout).await?;

                Ok(AsyncResult::Stdout(stdout))
            }
        }
    }

    fn create_shell_command(&self, lines: Option<String>) -> tokio::process::Command {
        // TODO: optimize: save ["sh", "-c", cmd] in hashmap to avoid reallocation
        let sh = vec!["sh", "-c", &self.command];

        let mut command = tokio::process::Command::new(sh[0]);
        command.args(&sh[1..]);
        if let Some(lines) = &lines {
            command.env("LINES", lines);
        }

        command
    }
}

/// Return error in case the exit status/code indicates failure, and include
/// stderr in error message.
async fn child_exited_successfully(
    exit_status: ExitStatus,
    stderr: &mut Option<tokio::process::ChildStderr>,
) -> Result<()> {
    if !exit_status.success() {
        // Read exit code
        let status_code_str = match exit_status.code() {
            Some(code) => Cow::Owned(code.to_string()),
            None => Cow::Borrowed("unknown"),
        };

        let stderr_str = match stderr {
            Some(stderr) => {
                // Read stderr
                let mut stderr_str = String::new();
                stderr.read_to_string(&mut stderr_str).await?;

                Cow::Owned(format!("stderr:\n{}", stderr_str))
            }
            None => Cow::Borrowed("unknown stderr"),
        };
        bail!(
            "Process exited with status code: {} and {}",
            status_code_str,
            stderr_str
        );
    }
    Ok(())
}

// TODO: update tests
#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test_executing_echo_command() -> Result<()> {
    //     let (_, rx) = std::sync::mpsc::channel();
    //     let echo_cmd = r#"echo "hello world""#.to_owned();
    //     let command: Command = echo_cmd.parse()?;
    //     let output_lines = command.capture_output(&rx)?;
    //     assert_eq!(output_lines, "hello world\n");
    //     Ok(())
    // }

    // #[test]
    // fn test_multiline_output() -> Result<()> {
    //     let (_, rx) = std::sync::mpsc::channel();
    //     let cmd = r#"printf "one\ntwo\n""#.to_owned();
    //     let command: Command = cmd.parse()?;
    //     let output_lines = command.capture_output(&rx)?;
    //     assert_eq!(output_lines, "one\ntwo\n");
    //     Ok(())
    // }

    // TODO: can't add env AND capture output right now
    // #[test]
    // fn test_adding_lines_env_variable() -> Result<()> {
    // 	Ok(())
    // }
}
