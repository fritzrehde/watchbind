use std::process::Command;
use std::io::Error;

// TODO: try &Vec<String>
// TODO: possibly replace all Strings with &str
// TODO: handle command failing
pub fn output_lines(command: &Vec<&str>) -> Result<Vec<String>, Error> {
	// execute command
	let output = Command::new(command[0])
		.args(&command[1..])
		.output()?;

	// get stdout from std::process:Output
	let lines = String::from_utf8(output.stdout).unwrap()
		.lines()
		.map(|s| s.to_string())
		.collect();
	Ok(lines)
}
