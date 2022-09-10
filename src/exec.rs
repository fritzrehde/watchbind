use std::process::Command;
use std::io::{Error, ErrorKind};

pub fn output_lines(cmd: &str) -> Result<Vec<String>, Error> {
	// execute command
	let command: Vec<&str> = vec!["sh", "-c", cmd];
	let output = Command::new(command[0])
		.args(&command[1..])
		.output()?;

	// get stdout
	let lines = String::from_utf8(output.stdout).unwrap()
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
