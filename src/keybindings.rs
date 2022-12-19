use crate::exec::{exec_blocking, exec_non_blocking};
use crate::state::State;
use crate::tui::{Event, RequestedAction};
use anyhow::{bail, Context, Result};
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use std::{
	collections::{HashMap, VecDeque},
	str::FromStr,
	sync::mpsc::Sender,
};

// TODO: split Operations into own file (in hidden sub folder)
// TODO: split Key into own file (in sub folder)

pub type Operations = VecDeque<Operation>;
pub type Keybindings = HashMap<Key, Operations>;
pub type KeybindingsRaw = HashMap<String, Vec<String>>;

#[derive(Hash, Eq, PartialEq)]
pub struct Key {
	code: KeyCode,
	modifiers: KeyModifiers,
}

// TODO: add support for goto nth line
#[derive(Clone)]
pub enum MoveCursor {
	Down(usize),
	Up(usize),
	First,
	Last,
}

#[derive(Clone)]
pub enum SelectOperation {
	Select,
	Unselect,
	Toggle,
	SelectAll,
	UnselectAll,
}

#[derive(Clone)]
pub struct Command {
	pub command: String,
	pub blocking: bool,
}

#[derive(Clone)]
pub enum Operation {
	Exit,
	Reload,
	MoveCursor(MoveCursor),
	SelectLine(SelectOperation),
	Execute(Command),
}

// TODO: return (&str, &str), deal with lifetime
// TODO: replace with nom
pub fn parse_str(s: &str) -> Result<(String, Vec<String>)> {
	let Some((key, operations)) = s.split_once(':') else {
		bail!("invalid format: expected \"KEY:OP[+OP]*\", found \"{}\"", s);
	};

	Ok((
		key.to_string(),
		// split on "+" and trim leading and trailing whitespace
		operations
			.split('+')
			.map(|op| op.trim().to_owned())
			.collect(),
	))
}

pub fn parse_raw(raw: KeybindingsRaw) -> Result<Keybindings> {
	raw
		.into_iter()
		.map(|(key, ops)| Ok((key.parse()?, operations_from_str(ops)?)))
		.collect()
}

// new and old have same key => keep new value
pub fn merge_raw(new: KeybindingsRaw, old: KeybindingsRaw) -> KeybindingsRaw {
	let mut merged = old.clone();
	merged.extend(new);
	merged
}

pub fn exec_operation(
	operation: &Operation,
	state: &mut State,
	event_tx: &Sender<Event>,
) -> Result<RequestedAction> {
	match operation {
		Operation::MoveCursor(MoveCursor::Down(steps)) => state.down(*steps),
		Operation::MoveCursor(MoveCursor::Up(steps)) => state.up(*steps),
		Operation::MoveCursor(MoveCursor::First) => state.first(),
		Operation::MoveCursor(MoveCursor::Last) => state.last(),
		Operation::SelectLine(SelectOperation::Select) => state.select(),
		Operation::SelectLine(SelectOperation::Unselect) => state.unselect(),
		Operation::SelectLine(SelectOperation::Toggle) => state.select_toggle(),
		Operation::SelectLine(SelectOperation::SelectAll) => state.select_all(),
		Operation::SelectLine(SelectOperation::UnselectAll) => state.unselect_all(),
		Operation::Reload => return Ok(RequestedAction::Reload),
		Operation::Exit => return Ok(RequestedAction::Exit),
		Operation::Execute(command) => {
			let lines = state.get_selected_lines();
			return Ok(if command.blocking {
				exec_blocking(&command.command, &lines, event_tx.clone());
				RequestedAction::Block
			} else {
				exec_non_blocking(&command.command, &lines)?;
				RequestedAction::Continue
			});
		}
	};
	Ok(RequestedAction::Continue)
}

impl FromStr for Operation {
	type Err = anyhow::Error;
	fn from_str(src: &str) -> Result<Operation> {
		// TODO: consider creating type "StepSize"
		let parse_steps = |steps: &str, src| {
			steps
				.parse()
				.with_context(|| format!("Invalid step size \"{steps}\" provided in keybinding: \"{src}\""))
		};
		Ok(match src.split_whitespace().collect::<Vec<&str>>()[..] {
			["exit"] => Operation::Exit,
			["reload"] => Operation::Reload,
			["down"] => Operation::MoveCursor(MoveCursor::Down(1)),
			["up"] => Operation::MoveCursor(MoveCursor::Up(1)),
			["down", steps] => Operation::MoveCursor(MoveCursor::Down(parse_steps(steps, src)?)),
			["up", steps] => Operation::MoveCursor(MoveCursor::Up(parse_steps(steps, src)?)),
			["first"] => Operation::MoveCursor(MoveCursor::First),
			["last"] => Operation::MoveCursor(MoveCursor::Last),
			["select"] => Operation::SelectLine(SelectOperation::Select),
			["unselect"] => Operation::SelectLine(SelectOperation::Unselect),
			["select-toggle"] => Operation::SelectLine(SelectOperation::Toggle),
			["select-all"] => Operation::SelectLine(SelectOperation::SelectAll),
			["unselect-all"] => Operation::SelectLine(SelectOperation::UnselectAll),
			_ => Operation::Execute(Command {
				command: src.to_string(),
				blocking: !src.contains("&"),
			}),
		})
	}
}

// TODO: turn into own type and implement FromStr trait
fn operations_from_str(ops: Vec<String>) -> Result<Operations> {
	ops
		.into_iter()
		.map(|op| Ok(Operation::from_str(&op)?))
		.collect()
}

impl Key {
	pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Key {
		Key { code, modifiers }
	}
}
fn parse_modifiers(s: &str) -> Result<KeyModifiers> {
	Ok(match s {
		"alt" => KeyModifiers::ALT,
		"ctrl" => KeyModifiers::CONTROL,
		invalid => bail!("Invalid key modifier provided in keybinding: {}", invalid),
	})
}

fn parse_code(s: &str) -> Result<(KeyModifiers, KeyCode)> {
	Ok(
		if s.len() == 1 {
			let c = s.chars().next().unwrap();
			let modifier = match c.is_uppercase() {
				true => KeyModifiers::SHIFT,
				false => KeyModifiers::NONE,
			};
			(modifier, KeyCode::Char(c))
		} else {
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
				invalid => bail!("Invalid key code provided in keybinding: {}", invalid),
			};
			(KeyModifiers::NONE, code)
	})
}

impl FromStr for Key {
	type Err = anyhow::Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (modifiers, code) = match s.split_once('+') {
			Some((s1, s2)) => {
				let mut mod1 = parse_modifiers(s1)?;
				let (mod2, code) = parse_code(s2)?;
				mod1.insert(mod2);
				(mod1, code)
			}
			None => parse_code(s)?,
		};
		Ok(Key { code, modifiers })
	}
}

pub fn default_raw() -> KeybindingsRaw {
	[
		("ctrl+c", vec!["exit"]),
		("q", vec!["exit"]),
		("r", vec!["reload"]),
		("space", vec!["select-toggle", "down"]),
		("v", vec!["select-toggle"]),
		("esc", vec!["unselect-all"]),
		("down", vec!["down"]),
		("up", vec!["up"]),
		("j", vec!["down"]),
		("k", vec!["up"]),
		("g", vec!["first"]),
		("G", vec!["last"]),
	]
	.into_iter()
	.map(|(key, commands)| {
		(
			key.to_string(),
			commands.into_iter().map(|cmd| cmd.to_string()).collect(),
		)
	})
	.collect()
}
