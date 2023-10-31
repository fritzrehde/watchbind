use crate::ui::{EnvVariables, InterruptSignal};
use anyhow::{bail, Result};
use std::{
    borrow::Cow,
    collections::HashMap,
    ops::Deref,
    process::{ExitStatus, Stdio},
    sync::Arc,
    time::Duration,
};
use tokio::process::Command as TokioCommand;
use tokio::sync::mpsc::Receiver;
use tokio::{io::AsyncReadExt, sync::Mutex};

// Type-States

#[derive(Clone)]
pub struct NonBlocking;
#[derive(Clone)]
pub struct Blocking;

#[derive(Clone)]
pub struct WithoutEnv;
#[derive(Clone)]
pub struct WithEnv {
    env_variables: Arc<Mutex<EnvVariables>>,
}

#[derive(Clone)]
pub struct NoOutput;
#[derive(Clone)]
pub struct WithOutput;

#[derive(Clone)]
pub struct NonInterruptible;
pub struct Interruptible {
    pub interrupt_rx: Receiver<InterruptSignal>,
}

// Advantages of the Type-State Builder Pattern:
// 1. We don't have any option/enum (an alternative configuration strategy)
// checking overhead at runtime.
// 2. We can guarantee that we handled all possible Command "variants"
// (combination of config options) that we use, at compile-time.
// 3. Arguably, this results in separated, cleaner code.

/// A Command offering customization of the blocking behaviour, the input
/// environment variables, whether the output is captured and whether the
/// execution can be interrupted. Utilizes the type-state builder pattern to
/// enforce these configurations at compile-time.
// #[derive(Clone)]
pub struct CommandBuilder<B = NonBlocking, E = WithoutEnv, O = NoOutput, I = NonInterruptible> {
    command: String,
    blocking: B,
    output: O,
    interruptible: I,
    env: E,
    tokio_command: TokioCommandBuilder,
}

// TODO: impl default trait so we don't have to duplicate this
impl CommandBuilder {
    pub fn new(command: String) -> Self {
        let sh = ["sh", "-c", &command];
        let mut process = TokioCommand::new(sh[0]);
        process.args(&sh[1..]);
        process.stdout(Stdio::null());
        process.stderr(Stdio::null());

        CommandBuilder {
            // TODO: i think we don't even need command anymore, just the tokiocommand
            command,
            blocking: NonBlocking,
            output: NoOutput,
            interruptible: NonInterruptible,
            env: WithoutEnv,
            tokio_command: Default::default(),
        }
    }
}

/// Since we can't save a tokio::process::Command permanently and just clone it
/// on new executions (it doesn't implement Clone), we store most of what's
/// necessary to contruct it.
#[derive(Default, Clone)]
struct TokioCommandBuilder {
    stdout: StdioClonable,
    stderr: StdioClonable,
}

// TODO: this should be known at compile-time as well, not have a match statement
#[derive(Default, Clone)]
enum StdioClonable {
    Piped,
    #[default]
    Null,
}

impl From<&StdioClonable> for Stdio {
    fn from(value: &StdioClonable) -> Self {
        match value {
            StdioClonable::Piped => Stdio::piped(),
            StdioClonable::Null => Stdio::null(),
        }
    }
}

// Provide methods for the builder pattern
impl<B, E, O, I> CommandBuilder<B, E, O, I> {
    pub fn blocking(mut self) -> CommandBuilder<Blocking, E, O, I> {
        // When we wait for the command to complete, we want to analyze the
        // exit status and stderr afterwards.
        self.tokio_command.stderr = StdioClonable::Piped;

        CommandBuilder {
            command: self.command,
            blocking: Blocking,
            output: self.output,
            interruptible: self.interruptible,
            env: self.env,
            tokio_command: self.tokio_command,
        }
    }

    pub fn with_env(
        self,
        env_variables: Arc<Mutex<EnvVariables>>,
    ) -> CommandBuilder<B, WithEnv, O, I> {
        CommandBuilder {
            command: self.command,
            blocking: self.blocking,
            output: self.output,
            interruptible: self.interruptible,
            env: WithEnv { env_variables },
            tokio_command: self.tokio_command,
        }
    }

    pub fn with_output(mut self) -> CommandBuilder<B, E, WithOutput, I> {
        // Required for obtaining stdout from child process.
        self.tokio_command.stdout = StdioClonable::Piped;

        CommandBuilder {
            command: self.command,
            blocking: self.blocking,
            output: WithOutput,
            interruptible: self.interruptible,
            env: self.env,
            tokio_command: self.tokio_command,
        }
    }

    pub fn interruptible(
        self,
        interrupt_rx: Receiver<InterruptSignal>,
    ) -> CommandBuilder<B, E, O, Interruptible> {
        CommandBuilder {
            command: self.command,
            blocking: self.blocking,
            output: self.output,
            interruptible: Interruptible { interrupt_rx },
            env: self.env,
            tokio_command: self.tokio_command,
        }
    }
}

impl<B, O, I> CommandBuilder<B, WithoutEnv, O, I> {
    async fn create_shell_command(&self) -> TokioCommand {
        // TODO: optimize: save ["sh", "-c", cmd] in hashmap to avoid reallocation
        let sh = ["sh", "-c", &self.command];

        let mut command = TokioCommand::new(sh[0]);

        command.args(&sh[1..]);
        command.stdout(&self.tokio_command.stdout);
        command.stderr(&self.tokio_command.stderr);

        command
    }
}

impl<B, O, I> CommandBuilder<B, WithEnv, O, I> {
    async fn create_shell_command(&self) -> TokioCommand {
        // TODO: optimize: save ["sh", "-c", cmd] in hashmap to avoid reallocation
        let sh = ["sh", "-c", &self.command];

        let mut command = TokioCommand::new(sh[0]);

        command.args(&sh[1..]);
        command.stdout(&self.tokio_command.stdout);
        command.stderr(&self.tokio_command.stderr);

        let env_variables: HashMap<_, _> = self.env.env_variables.lock().await.deref().into();
        command.envs(env_variables);

        command
    }
}

// TODO: remove code duplication
// TODO: see where we can make it even more generic

impl CommandBuilder<NonBlocking, WithoutEnv, NoOutput, NonInterruptible> {
    pub async fn execute(&self) -> Result<()> {
        self.create_shell_command().await.spawn()?;

        // create_shell_command(&self.command)
        //     // We only need stderr in case of an error, stdout can be ignored
        //     .stdout(Stdio::null())
        //     .stderr(Stdio::piped())
        //     .spawn()?;

        Ok(())
    }
}

impl CommandBuilder<NonBlocking, WithEnv, NoOutput, NonInterruptible> {
    pub async fn execute(&self) -> Result<()> {
        self.create_shell_command().await.spawn()?;

        // let env_variables = self.env.env_variables.lock().await.deref().into();
        // create_shell_command_with_env(&self.command, env_variables)
        //     // We only need stderr in case of an error, stdout can be ignored
        //     .stdout(Stdio::null())
        //     .stderr(Stdio::piped())
        //     .spawn()?;

        Ok(())
    }
}

impl CommandBuilder<Blocking, WithEnv, NoOutput, NonInterruptible> {
    pub async fn execute(&self) -> Result<()> {
        let mut child = self.create_shell_command().await.spawn()?;

        // let env_variables = self.env.env_variables.lock().await.deref().into();
        // let mut child = create_shell_command_with_env(&self.command, env_variables)
        //     // We only need stderr in case of an error, stdout can be ignored
        //     .stdout(Stdio::null())
        //     .stderr(Stdio::piped())
        //     .spawn()?;

        let exit_status = child.wait().await?;
        assert_child_exited_successfully(exit_status, &mut child.stderr).await?;

        Ok(())
    }
}

impl CommandBuilder<Blocking, WithoutEnv, WithOutput, NonInterruptible> {
    pub async fn execute(&self) -> Result<String> {
        let mut child = self.create_shell_command().await.spawn()?;

        // let mut child = create_shell_command(&self.command)
        //     // Keep both stdout and stderr
        //     .stdout(Stdio::piped())
        //     .stderr(Stdio::piped())
        //     .spawn()?;

        let exit_status = child.wait().await?;
        assert_child_exited_successfully(exit_status, &mut child.stderr).await?;

        // Read stdout
        let mut stdout = String::new();
        child.stdout.unwrap().read_to_string(&mut stdout).await?;

        Ok(stdout)
    }
}

impl CommandBuilder<Blocking, WithEnv, WithOutput, NonInterruptible> {
    pub async fn execute(&self) -> Result<String> {
        let mut child = self.create_shell_command().await.spawn()?;

        // let env_variables = self.env.env_variables.lock().await.deref().into();
        // let mut child = create_shell_command_with_env(&self.command, env_variables)
        //     // Keep both stdout and stderr
        //     .stdout(Stdio::piped())
        //     .stderr(Stdio::piped())
        //     .spawn()?;

        let exit_status = child.wait().await?;
        assert_child_exited_successfully(exit_status, &mut child.stderr).await?;

        // Read stdout
        let mut stdout = String::new();
        child.stdout.unwrap().read_to_string(&mut stdout).await?;

        Ok(stdout)
    }
}

/// Encodes whether a command's execution was interrupted, or the stdout if it
/// ran to completion.
pub enum ExecutionResult {
    Stdout(String),
    Interrupted,
}

// TODO: find better name
/// The async call was woken up due to this reason.
pub enum WasWoken {
    ReceivedInterrupt,
    ChannelClosed,
}

impl CommandBuilder<Blocking, WithEnv, WithOutput, Interruptible> {
    pub async fn execute(&mut self) -> Result<ExecutionResult> {
        let mut child = self.create_shell_command().await.spawn()?;

        // let env_variables = self.env.env_variables.lock().await.deref().into();

        // let mut child = create_shell_command_with_env(&self.command, env_variables)
        //     // Keep both stdout and stderr
        //     .stdout(Stdio::piped())
        //     .stderr(Stdio::piped())
        //     .spawn()?;

        tokio::select! {
            _ = self.interruptible.interrupt_rx.recv() => {
                child.kill().await?;
                Ok(ExecutionResult::Interrupted)
            },
            exit_status = child.wait() => {
                assert_child_exited_successfully(exit_status?, &mut child.stderr).await?;

                // Read stdout
                let mut stdout = String::new();
                // TODO: remove unwrap()
                child.stdout.unwrap().read_to_string(&mut stdout).await?;

                Ok(ExecutionResult::Stdout(stdout))
            }
        }
    }
}

impl<B, E, O> CommandBuilder<B, E, O, Interruptible> {
    /// Waits indefinitely for an interrupt signal.
    pub async fn wait_for_interrupt(&mut self) -> WasWoken {
        match self.interruptible.interrupt_rx.recv().await {
            Some(InterruptSignal) => WasWoken::ReceivedInterrupt,
            None => WasWoken::ChannelClosed,
        }
    }

    /// Waits for an interrupt signal up to a given timeout duration.
    pub async fn wait_for_interrupt_within_timeout(&mut self, timeout: Duration) -> WasWoken {
        match tokio::time::timeout(timeout, self.interruptible.interrupt_rx.recv()).await {
            Ok(None) => WasWoken::ChannelClosed,
            Ok(Some(InterruptSignal)) | Err(_) => WasWoken::ReceivedInterrupt,
        }
    }
}

/// Return error in case the exit status/code indicates failure, and include
/// stderr in error message.
async fn assert_child_exited_successfully(
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
            "Process exited with status code: {} and stderr: {}",
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
