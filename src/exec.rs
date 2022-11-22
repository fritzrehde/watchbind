use std::io::{Error, ErrorKind};
use std::process::Command;

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

// TODO: group cmd and background into one struct
// TODO: optimize: save ["sh", "-c", cmd] in hashmap to avoid reallocation
pub fn run_line(cmd: &str, line: &str, background: bool) -> Result<(), Error> {
	// execute command
	let sh: Vec<&str> = vec!["sh", "-c", cmd];
	let mut cmd = Command::new(sh[0]);
	let cmd = cmd
		.env("LINE", line) // provide selected line as environment variable
		.args(&sh[1..]);

	if background {
		cmd.spawn()?;
	} else {
		let output = cmd.output()?;
		// handle command error
		if !output.status.success() {
			let stderr = String::from_utf8(output.stderr).unwrap();
			return Err(Error::new(ErrorKind::Other, stderr));
		}
	}
	Ok(())
}
