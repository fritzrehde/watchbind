use crossterm::event::KeyCode::{self, *};
use itertools::{Itertools, chain};
use std::{
	collections::HashMap,
	io::{self, Error, ErrorKind},
	process,
	str::FromStr,
};
use crate::events::Events;
use crate::keys::Command::*;

pub type Keybindings = HashMap<KeyCode, Command>;
pub type KeybindingsRaw = HashMap<String, String>;

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

// TODO: handle duplicates
// TODO: merge two map() into one
pub fn parse_str(bindings: String) -> KeybindingsRaw {
	bindings.split(",")
		.filter(|s| s.matches(":").count() == 1) // only keep bindings with exactly one ":"
		.map(|s| s.split(":").collect_tuple().unwrap())
		.map(|(k, v)| (k.to_string(), v.to_string()))
		.collect()
}

pub fn parse_raw(raw: KeybindingsRaw) -> Keybindings {
	raw.into_iter()
		.map(|(key, cmd)| {
			(
				keycode_from_str(&key).unwrap(),
				Command::from_str(&cmd).unwrap(),
			)
		})
		.collect()
}

// new and old have same key => keep new value
pub fn merge_raw(new: KeybindingsRaw, old: KeybindingsRaw) -> KeybindingsRaw {
	let mut merged = old.clone();
	merged.extend(new);
	merged
	// chain!(new.into_iter(), old.into_iter())
	// 	.unique_by(|(k, _)| k) 
	// 	// .unique()
	// 	.collect()
}

pub fn handle_key(
	key: KeyCode,
	keybindings: &Keybindings,
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
					// TODO: move to exec module
					// TODO: instantly reload afterwards
					// execute command
					let command: Vec<&str> = vec!["sh", "-c", cmd];
					let line = events.get_selected_line().unwrap_or(""); // no line selected => LINE=""
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
		Ok(match src {
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

pub fn default_keybindingsraw() -> KeybindingsRaw {
	[
		("q", "exit"),
		("esc", "unselect"),
		("down", "next"),
		("up", "previous"),
		("j", "next"),
		("k", "previous"),
		("g", "first"),
		("G", "last")
	]
	.into_iter()
	.map(|(k, v)| (k.to_string(), v.to_string()))
	.collect()
}
