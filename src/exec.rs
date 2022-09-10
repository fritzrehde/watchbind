use std::process::Command;
use std::io::{Error, ErrorKind};

// TODO: try &Vec<String>
// TODO: possibly replace all Strings with &str
// TODO: handle command failing
pub fn output_lines(command: &Vec<&str>) -> Result<Vec<String>, Error> {
	// execute command
	let output = Command::new(command[0])
		.args(&command[1..])
		.output()?;

	// get stdout
	let lines = String::from_utf8(output.stdout).unwrap()
		.lines()
		.map(|s| s.to_string())
		.collect();

	// TODO: possibly get stderr instead of stdout for failure
	match output.status.success() {
		true => Ok(lines),
		false => {
			let error_msg = match output.status.code() {
				Some(code) => format!("Command ... failed with error code {}", code),
				None => format!("Command ... terminated by signal"),
			};
			Err(Error::new(ErrorKind::Other, error_msg))
		}
	}

}
