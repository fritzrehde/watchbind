use anyhow::{bail, Result};
use std::process::{self, Output};

#[derive(Clone)]
pub struct Command {
	command: String,
	is_blocking: bool,
}

impl Command {
	pub fn new(mut command: String) -> Self {
		let is_blocking = !command.ends_with(" &");
		if command.ends_with(" &") {
			command.truncate(command.len() - " &".len());
		}
		Self {
			command,
			is_blocking,
		}
	}

	pub fn is_blocking(&self) -> bool {
		self.is_blocking
	}

	// TODO: merge into execute function
	pub fn capture_output(&self) -> Result<Vec<String>> {
		let output = self.shell_cmd(None).output()?;

		// TODO: add support for blocking and non-blocking
		let lines = String::from_utf8(output.stdout.clone())
			.unwrap()
			.lines()
			.map(|s| s.to_string())
			.collect();

		check_stderr(output)?;
		Ok(lines)
	}

	pub fn execute(&self, lines: Option<String>) -> Result<()> {
		let mut cmd = self.shell_cmd(lines);
		if self.is_blocking {
			check_stderr(cmd.output()?)?
		} else {
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

fn check_stderr(output: Output) -> Result<()> {
	if !output.status.success() {
		bail!(String::from_utf8(output.stderr).unwrap());
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_executing_echo_command() -> Result<()> {
		let echo_cmd = r#"echo "hello world""#.to_owned();
		let output_lines = Command::new(echo_cmd).capture_output()?;
		assert_eq!(output_lines, vec!["hello world".to_owned()]);
		Ok(())
	}

	// TODO: can't add env AND capture output right now
	// #[test]
	// fn test_adding_lines_env_variable() -> Result<()> {
	// 	Ok(())
	// }
}
