use std::process::{Command, Output};
use std::io::Error;

pub fn output_lines<'a>(command: impl Iterator<Item = &'a String>) -> Result<Vec<String>, Error> {
// pub fn output_lines(command: &str) -> Result<Vec<String>, Error> {
	let output = exec_command(command)?;
	let lines = String::from_utf8(output.stdout).unwrap()
		.lines().map(|s| s.to_string())
		.collect();
	Ok(lines)
}

// TODO: inefficient to create new iterator every time command is executed
pub fn exec_command<'a>(args: impl Iterator<Item = &'a String>) -> Result<Output, Error> {
// pub fn exec_command(command: &str) -> Result<Output, Error> {
	// let mut args = command.split_whitespace();
	Command::new(args.next().expect("Command not empty")) // TODO: handle empty command
		.args(args)
		.output()
}
