use crate::ui::Event;
use anyhow::{anyhow, Result};
use std::process::{self, Output};
use std::{sync::mpsc::Sender, thread};

#[derive(Clone)]
pub struct Command {
	command: String,
	// TODO: turn Sender<Event> into own type
	blocking: Option<Sender<Event>>,
}

impl Command {
	pub fn new(command: String) -> Self {
		Self {
			command,
			blocking: None,
		}
	}

	pub fn block(&mut self, tx: Sender<Event>) {
		self.blocking = Some(tx);
	}

	pub fn is_blocking(&self) -> bool {
		self.blocking.is_some()
	}

	// TODO: merge into execute function
	pub fn capture_output(self) -> Result<Vec<String>> {
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
		match &self.blocking {
			None => {
				cmd.spawn()?;
			},
			Some(event_tx) => {
				let tx = event_tx.clone();
				thread::spawn(move || {
					let mut exec = move || {
						check_stderr(cmd.output()?)
					};
					tx.send(Event::Unblock(exec())).unwrap();
				});
			},
		};
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
	match output.status.success() {
		false => Err(anyhow!(String::from_utf8(output.stderr).unwrap())),
		true => Ok(()),
	}
}
