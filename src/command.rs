use std::process::Command;

pub fn output_lines(command: &str) -> Vec<String> {
	let output = Command::new(command).output().unwrap();
	String::from_utf8(output.stdout).unwrap()
		.lines().map(|s| s.to_string())
		.collect::<Vec<String>>()
}
