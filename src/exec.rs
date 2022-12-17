use crate::keybindings::Command as CCommand;
use anyhow::{bail, Result};
use std::process::Command;

pub fn output_lines(cmd: &str) -> Result<Vec<String>> {
	// execute command
	let command = vec!["sh", "-c", cmd];
	let output = Command::new(command[0]).args(&command[1..]).output()?;

	// get stdout
	let lines = String::from_utf8(output.stdout)
		.unwrap()
		.lines()
		.map(|s| s.to_string())
		.collect();

	// handle command error
	if output.status.success() {
		Ok(lines)
	} else {
		bail!(String::from_utf8(output.stderr).unwrap())
	}
}

// TODO: optimize: save ["sh", "-c", cmd] in hashmap to avoid reallocation
pub fn run_lines(cmd: &CCommand, lines: &str) -> Result<()> {
	// execute command
	let sh = vec!["sh", "-c", &cmd.command];
	let mut command = Command::new(sh[0]);

	// provide selected line as environment variable
	command.env("LINES", lines).args(&sh[1..]);

	if cmd.background {
		command.spawn()?;
	} else {
		let output = command.output()?;
		// handle command error
		if !output.status.success() {
			bail!(String::from_utf8(output.stderr).unwrap());
		}
	}
	Ok(())
}
