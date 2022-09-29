use crate::events::Events;
use crate::exec;
use crate::keys::Command::*;
use crossterm::event::KeyCode::{self, *};
use itertools::Itertools;
use std::{
	collections::HashMap,
	io::{Error, ErrorKind},
	str::FromStr,
};

pub type Keybindings = HashMap<KeyCode, Command>;
pub type KeybindingsRaw = HashMap<String, String>;

// TODO: add reload command
#[derive(Debug, Clone)]
pub enum Command {
	Exit,
	Unselect,
	Next,
	Previous,
	First,
	Last,
	Execute(String),
}

pub fn parse_key_val(s: &str) -> Result<(String, String), Error> {
	let pos = s
		.find(':')
		.ok_or_else(|| Error::new(
			ErrorKind::Other,
			format!("invalid KEY:value: no `:` found in `{}`", s)
		))?;
	Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

pub fn parse_raw(raw: KeybindingsRaw) -> Result<Keybindings, Error> {
	raw
		.into_iter()
		.map(|(key, cmd)| Ok((keycode_from_str(&key)?, Command::from_str(&cmd)?)))
		.collect()
}

// new and old have same key => keep new value
pub fn merge_raw(new: KeybindingsRaw, old: KeybindingsRaw) -> KeybindingsRaw {
	let mut merged = old.clone();
	merged.extend(new);
	merged
}

pub fn handle_key(
	key: KeyCode,
	keybindings: &Keybindings,
	events: &mut Events,
) -> Result<bool, Error> {
	match keybindings.get(&key) {
		Some(binding) => {
			match binding {
				Exit => return Ok(false),
				Unselect => events.unselect(),
				Next => events.next(),
				Previous => events.previous(),
				First => events.first(),
				Last => events.last(),
				// TODO: instantly reload afterwards
				Execute(cmd) => exec::run_selected_line(&cmd, events)?,
			};
		}
		None => {} // do nothing, since key has no binding
	};
	Ok(true)
}

pub fn default_raw() -> KeybindingsRaw {
	let keybindings = "q:exit,esc:unselect,down:next,up:previous,j:next,k:previous,g:first,G:last";
	parse_str(keybindings.to_string())
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

impl FromStr for Command {
	type Err = Error;
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
