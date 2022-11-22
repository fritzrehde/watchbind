use crate::events::Events;
use crate::exec;
use crossterm::event::KeyCode::{self, *};
use std::{
	collections::HashMap,
	io::{Error, ErrorKind},
	str::FromStr,
	sync::mpsc,
};

pub type Keybindings = HashMap<KeyCode, Operations>;
pub type KeybindingsRaw = HashMap<String, String>;
pub type Operations = Vec<Operation>;

#[derive(Clone)]
pub enum Operation {
	Exit,
	Reload,
	Unselect,
	Next,
	Previous,
	First,
	Last,
	// execute as background process or wait for termination
	Execute { background: bool, command: String },
}

pub fn parse_str(s: &str) -> Result<(String, String), Error> {
	let pos = s.find(':').ok_or_else(|| {
		Error::new(
			ErrorKind::Other,
			format!("invalid KEY:value: no \":\" found in \"{}\"", s),
		)
	})?;
	Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

pub fn parse_raw(raw: KeybindingsRaw) -> Result<Keybindings, Error> {
	raw
		.into_iter()
		.map(|(key, cmd)| Ok((keycode_from_str(&key)?, operations_from_str(&cmd)?)))
		.collect()
}

// new and old have same key => keep new value
pub fn merge_raw(new: KeybindingsRaw, old: KeybindingsRaw) -> KeybindingsRaw {
	let mut merged = old.clone();
	merged.extend(new);
	merged
}

fn exec_operation(
	operation: &Operation,
	events: &mut Events,
	thread_channel: &mpsc::Sender<bool>,
) -> Result<bool, Error> {
	match operation {
		Operation::Unselect => events.unselect(),
		Operation::Next => events.next(),
		Operation::Previous => events.previous(),
		Operation::First => events.first(),
		Operation::Last => events.last(),
		// reload input by waking up thread
		Operation::Execute {
			background,
			command,
		} => {
			let line = events.get_selected_line().unwrap_or(""); // no line selected => LINE=""
			exec::run_line(&command, line, *background)?
		}
		Operation::Reload => thread_channel.send(true).unwrap(),
		Operation::Exit => return Ok(false),
	};
	Ok(true)
}

pub fn handle_key(
	key: KeyCode,
	keybindings: &Keybindings,
	events: &mut Events,
	// TODO: convert to Sender<()>
	thread_channel: &mpsc::Sender<bool>,
) -> Result<bool, Error> {
	if let Some(operations) = keybindings.get(&key) {
		for op in operations {
			if let false = exec_operation(op, events, thread_channel)? {
				// exit was called => program should be stopped
				return Ok(false);
			}
		}
	}
	Ok(true)
}


impl FromStr for Operation {
	type Err = Error;
	fn from_str(src: &str) -> Result<Operation, Self::Err> {
		Ok(match src {
			"exit" => Operation::Exit,
			"reload" => Operation::Reload,
			"unselect" => Operation::Unselect,
			"next" => Operation::Next,
			"previous" => Operation::Previous,
			"first" => Operation::First,
			"last" => Operation::Last,
			// TODO: remove " &" from cmd
			cmd => Operation::Execute {
				background: cmd.contains(" &"),
				command: cmd.to_string(),
			},
		})
	}
}

fn operations_from_str(s: &str) -> Result<Vec<Operation>, Error> {
	s.split('+').map(|s| Ok(Operation::from_str(s)?)).collect()
}

// TODO: add modifiers
fn keycode_from_str(s: &str) -> Result<KeyCode, Error> {
	let key = match s {
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
		invalid => {
			return Err(Error::new(
				ErrorKind::Other,
				format!("Invalid key provided in keybinding: {}", invalid),
			))
		}
	};
	Ok(key)
}

// TODO: idea: parse from file instead of hardcoded
pub fn default_raw() -> KeybindingsRaw {
	[
		("q", "exit"),
		("r", "reload"),
		("esc", "unselect"),
		("down", "next"),
		("up", "previous"),
		("j", "next"),
		("k", "previous"),
		("g", "first"),
		("G", "last"),
	]
	.into_iter()
	.map(|(k, v)| (k.to_string(), v.to_string()))
	.collect()
}
