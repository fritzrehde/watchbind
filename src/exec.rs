use std::io::{Error, ErrorKind};
use std::process::Command;
use crate::events::Events;

pub fn output_lines(cmd: &str) -> Result<Vec<String>, Error> {
	// execute command
	let command: Vec<&str> = vec!["sh", "-c", cmd];
	let output = Command::new(command[0]).args(&command[1..]).output()?;

	// get stdout
	let lines = String::from_utf8(output.stdout)
		.unwrap()
		.lines()
		.map(|s| s.to_string())
		.collect();

	// handle command error
	match output.status.success() {
		true => Ok(lines),
		false => {
			let stderr = String::from_utf8(output.stderr).unwrap();
			Err(Error::new(ErrorKind::Other, stderr))
		}
	}
}

// TODO: optimize: save ["sh", "-c", cmd] in hashmap to avoid reallocation
pub fn run_selected_line(cmd: &str, events: &mut Events) -> Result<(), Error> {
	// execute command
	let command: Vec<&str> = vec!["sh", "-c", cmd];
	let line = events.get_selected_line().unwrap_or(""); // no line selected => LINE=""
	let output = Command::new(command[0])
		.env("LINE", line) // provide selected line as environment variable
		.args(&command[1..])
		.output()?;

	// handle command error
	match output.status.success() {
		true => Ok(()),
		false => {
			let stderr = String::from_utf8(output.stderr).unwrap();
			return Err(Error::new(ErrorKind::Other, stderr));
		},
	}
}
