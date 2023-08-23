use anyhow::{bail, Error, Result};
use core::time::Duration;
use parse_display::Display;
use std::{
    process::{self, Output, Stdio},
    str::FromStr,
    sync::mpsc::Receiver,
    thread,
};

#[derive(Clone, Display, PartialEq, PartialOrd, Eq, Ord)]
#[display("{command}")]
pub struct Command {
    // TODO: remove pub
    pub command: String,
    is_blocking: bool,
}

// enum Event {
// 	Reload,
// 	OutputLines(Result<Vec<String>>),
// }

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

impl Command {
    pub fn is_blocking(&self) -> bool {
        self.is_blocking
    }

    // pub fn capture_output(&self, reload_rx: &Receiver<()>) -> Result<Vec<String>> {
    pub fn capture_output(&self, reload_rx: &Receiver<()>) -> Result<String> {
        // let mut cmd = self.shell_cmd(None);
        // let mut child = cmd.stdout(Stdio::piped());

        loop {
            let mut child = self.shell_cmd(None).stdout(Stdio::piped()).spawn()?;

            // let (tx, rx) = mpsc::sync_channel(1);

            // thread::spawn(|| {
            // 	reload_rx.recv().unwrap();
            // 	tx.clone().send(Event::Reload).unwrap();
            // });

            // thread::spawn(move || {
            // 	let mut exec = || {
            // 		let output = child.spawn()?.wait_with_output()?;
            // 		check_stderr(&output)?;
            // 		let lines = String::from_utf8(output.stdout)?
            // 			.lines()
            // 			.map(str::to_string)
            // 			.collect();
            // 		Ok(lines)
            // 	};
            // 	tx.clone().send(Event::OutputLines(exec())).unwrap();
            // });

            // TODO: remove busy waiting by creating two threads that send the same event and handle that
            // busy wait for reload signal or child process finishing
            loop {
                if reload_rx.try_recv().is_ok() {
                    child.kill().ok();
                    break;
                }
                if let Ok(Some(_)) = child.try_wait() {
                    let output = child.wait_with_output()?;
                    check_stderr(&output)?;
                    return Ok(String::from_utf8(output.stdout)?);
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

    pub fn execute(&self, lines: Option<String>) -> Result<()> {
        let mut cmd = self.shell_cmd(lines);
        if self.is_blocking {
            check_stderr(&cmd.output()?)?
        } else {
            // TODO: documentation states that calling wait is advised to release resources
            cmd.spawn()?;
        }
        Ok(())
    }

    fn shell_cmd(&self, lines: Option<String>) -> process::Command {
        // TODO: optimize: save ["sh", "-c", cmd] in hashmap to avoid reallocation
        let sh = vec!["sh", "-c", &self.command];
        let mut command = process::Command::new(sh[0]);
        command.args(&sh[1..]);
        if let Some(lines) = &lines {
            command.env("LINES", lines);
        }
        command
    }
}

fn check_stderr(output: &Output) -> Result<()> {
    if !output.status.success() {
        let status_code_str = match output.status.code() {
            Some(code) => code.to_string(),
            None => "unknown".to_owned(),
        };
        bail!(
            "Process exited with status code: {} and stderr:\n{}",
            status_code_str,
            String::from_utf8(output.stderr.clone())?
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executing_echo_command() -> Result<()> {
        let (_, rx) = std::sync::mpsc::channel();
        let echo_cmd = r#"echo "hello world""#.to_owned();
        let command: Command = echo_cmd.parse()?;
        let output_lines = command.capture_output(&rx)?;
        assert_eq!(output_lines, "hello world\n");
        Ok(())
    }

    #[test]
    fn test_multiline_output() -> Result<()> {
        let (_, rx) = std::sync::mpsc::channel();
        let cmd = r#"printf "one\ntwo\n""#.to_owned();
        let command: Command = cmd.parse()?;
        let output_lines = command.capture_output(&rx)?;
        assert_eq!(output_lines, "one\ntwo\n");
        Ok(())
    }

    // TODO: can't add env AND capture output right now
    // #[test]
    // fn test_adding_lines_env_variable() -> Result<()> {
    // 	Ok(())
    // }
}
