use anyhow::{bail, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::str::FromStr;

#[derive(Hash, Eq, PartialEq)]
pub struct Key(KeyEvent);

impl Key {
	pub fn new(event: KeyEvent) -> Self {
		Self(event)
	}
}

impl FromStr for Key {
	type Err = anyhow::Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let event = match s.split_once('+') {
			Some((s1, s2)) => {
				let mut event = parse_code(s2)?;
				event.modifiers.insert(parse_modifier(s1)?);
				event
			}
			None => parse_code(s)?,
		};
		Ok(Key(event))
	}
}

fn parse_modifier(s: &str) -> Result<KeyModifiers> {
	Ok(match s {
		"alt" => KeyModifiers::ALT,
		"ctrl" => KeyModifiers::CONTROL,
		invalid => bail!("Invalid key modifier provided in keybinding: {}", invalid),
	})
}

fn parse_code(s: &str) -> Result<KeyEvent> {
	let code = match s {
		"esc" => KeyCode::Esc,
		"enter" => KeyCode::Enter,
		"left" => KeyCode::Left,
		"right" => KeyCode::Right,
		"up" => KeyCode::Up,
		"down" => KeyCode::Down,
		"home" => KeyCode::Home,
		"end" => KeyCode::End,
		"pageup" => KeyCode::PageUp,
		"pagedown" => KeyCode::PageDown,
		"backtab" => KeyCode::BackTab,
		"backspace" => KeyCode::Backspace,
		"del" => KeyCode::Delete,
		"delete" => KeyCode::Delete,
		"insert" => KeyCode::Insert,
		"ins" => KeyCode::Insert,
		"f1" => KeyCode::F(1),
		"f2" => KeyCode::F(2),
		"f3" => KeyCode::F(3),
		"f4" => KeyCode::F(4),
		"f5" => KeyCode::F(5),
		"f6" => KeyCode::F(6),
		"f7" => KeyCode::F(7),
		"f8" => KeyCode::F(8),
		"f9" => KeyCode::F(9),
		"f10" => KeyCode::F(10),
		"f11" => KeyCode::F(11),
		"f12" => KeyCode::F(12),
		"space" => KeyCode::Char(' '),
		"tab" => KeyCode::Tab,
		c if c.len() == 1 => KeyCode::Char(c.chars().next().unwrap()),
		invalid => bail!("Invalid key code provided in keybinding: {}", invalid),
	};
	Ok(KeyEvent::from(code))
}
