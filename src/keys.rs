use crate::events::Events;
use crate::keys::Command::*;
use crossterm::event::KeyCode::{self, *};
use itertools::Itertools;
use std::{
	collections::HashMap,
	io::{self, Error, ErrorKind},
	process,
	str::FromStr,
};

const DEFAULT_BINDINGS: &str = "q:exit,esc:unselect,down:next,up:previous,j:next,k:previous,g:first,G:last";

// TODO: add reload command
#[derive(Debug)]
pub enum Command {
	Exit,
	Unselect,
	Next,
	Previous,
	First,
	Last,
	Execute(String),
}

pub fn parse_bindings(bindings: &str) -> io::Result<HashMap<KeyCode, Command>> {
	// TODO: handle duplicates
	Ok(
		format!("{},{}", DEFAULT_BINDINGS, bindings) // TODO: handle empty "bindings"
			.split(",")
			.filter(|s| s.matches(":").count() == 1) // only keep bindings with exactly one ":"
			.map(|s| s.split(":").collect_tuple().unwrap())
			.map(|(key, cmd)| {
				(
					keycode_from_str(key).unwrap(),
					Command::from_str(cmd).unwrap(),
				)
			})
			.collect(),
	)
}

pub fn handle_key(
	key: KeyCode,
	keybindings: &HashMap<KeyCode, Command>,
	events: &mut Events,
) -> Result<bool, io::Error> {
	match keybindings.get(&key) {
		Some(binding) => {
			match binding {
				Exit => return Ok(false),
				Unselect => events.unselect(),
				Next => events.next(),
				Previous => events.previous(),
				First => events.first(),
				Last => events.last(),
				Execute(cmd) => {
					// TODO: instantly reload afterwards
					match events.get_selected_line() {
						Some(line) => {
							// execute command
							let command: Vec<&str> = vec!["sh", "-c", cmd];
							let output = process::Command::new(command[0])
								.env("LINE", line) // provide selected line as environment variable
								.args(&command[1..])
								.output()?;

							// handle command error
							if !output.status.success() {
								let stderr = String::from_utf8(output.stderr).unwrap();
								return Err(Error::new(ErrorKind::Other, stderr));
							}
						}
						None => {} // no line selected, so do nothing
					}
				}
			};
		}
		None => {} // do nothing, since key has no binding
	};
	Ok(true)
}

// TODO: add modifiers
fn keycode_from_str(input: &str) -> Result<KeyCode, ()> {
	let key = match input {
		"esc" => Esc,
		"enter" => Enter,
		"left" => Left,
		"right" => Right,
		"up" => Up,
		"down" => Down,
		"home" => Home,
		"end" => End,
		"pageup" => PageUp,
		"pagedown" => PageDown,
		"backtab" => BackTab,
		"backspace" => Backspace,
		"del" => Delete,
		"delete" => Delete,
		"insert" => Insert,
		"ins" => Insert,
		"f1" => F(1),
		"f2" => F(2),
		"f3" => F(3),
		"f4" => F(4),
		"f5" => F(5),
		"f6" => F(6),
		"f7" => F(7),
		"f8" => F(8),
		"f9" => F(9),
		"f10" => F(10),
		"f11" => F(11),
		"f12" => F(12),
		"space" => Char(' '),
		"tab" => Tab,
		c if c.len() == 1 => Char(c.chars().next().unwrap()),
		_ => return Err(()),
	};
	Ok(key)
}

impl FromStr for Command {
	type Err = ();
	fn from_str(src: &str) -> Result<Command, Self::Err> {
		Ok(match src.to_lowercase().as_str() {
			"exit" => Exit,
			"unselect" => Unselect,
			"next" => Next,
			"previous" => Previous,
			"first" => First,
			"last" => Last,
			cmd => Execute(cmd.to_string()),
		})
	}
}
