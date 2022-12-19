use crate::tui::Event;
use anyhow::{anyhow, Result};
use std::process::Command;
use std::{sync::mpsc::Sender, thread};

fn shell_cmd(cmd: &str) -> Command {
	// TODO: optimize: save ["sh", "-c", cmd] in hashmap to avoid reallocation
	let sh = vec!["sh", "-c", cmd];
	let mut command = Command::new(sh[0]);
	command.args(&sh[1..]);
	command
}

fn shell_cmd_lines(cmd: &str, lines: &str) -> Command {
	let mut command = shell_cmd(cmd);
	command.env("LINES", lines);
	command
}

pub fn output_lines(cmd: &str) -> Result<Vec<String>> {
	let output = shell_cmd(cmd).output()?;

	let lines = String::from_utf8(output.stdout)
		.unwrap()
		.lines()
		.map(|s| s.to_string())
		.collect();

	match output.status.success() {
		true => Ok(lines),
		false => Err(anyhow!(String::from_utf8(output.stderr).unwrap())),
	}
}

pub fn exec_non_blocking(cmd: &str, lines: &str) -> Result<()> {
	shell_cmd_lines(cmd, lines).spawn()?;
	Ok(())
}

fn capture_output(command: &mut Command) -> Result<()> {
	let output = command.output()?;
	match output.status.success() {
		false => Err(anyhow!(String::from_utf8(output.stderr).unwrap())),
		true => Ok(()),
	}
}

pub fn exec_blocking(cmd: &str, lines: &str, event_tx: Sender<Event>) {
	let mut command = shell_cmd_lines(cmd, lines);
	thread::spawn(move || {
		event_tx
			.send(Event::Unblock(capture_output(&mut command)))
			.unwrap();
	});
}
