mod key;
mod operations;

pub use key::Key;
pub use operations::{Operation, Operations};
pub type Keybindings = HashMap<Key, Operations>;
pub type KeybindingsRaw = HashMap<String, Vec<String>>;

use crate::ui::Event;
use anyhow::{bail, Result};
use std::{collections::HashMap, sync::mpsc::Sender};

// TODO: return (&str, &str), deal with lifetime
// TODO: replace with nom
pub fn parse_str(s: &str) -> Result<(String, Vec<String>)> {
	let Some((key, operations)) = s.split_once(':') else {
		bail!("invalid format: expected \"KEY:OP[+OP]*\", found \"{}\"", s);
	};

	Ok((
		key.to_string(),
		operations
			.split('+')
			.map(|op| op.trim().to_owned())
			.collect(),
	))
}

pub fn parse_raw(raw: KeybindingsRaw) -> Result<Keybindings> {
	raw
		.into_iter()
		// .map(|(key, ops)| Ok((key.parse()?, operations_from_str(ops)?)))
		.map(|(key, ops)| Ok((key.parse()?, Operations::from_vec(ops)?)))
		.collect()
}

// new and old have same key => keep new value
pub fn merge_raw(new: KeybindingsRaw, old: KeybindingsRaw) -> KeybindingsRaw {
	let mut merged = old.clone();
	merged.extend(new);
	merged
}

pub fn add_event_tx(k: Keybindings, event_tx: &Sender<Event>) -> Keybindings {
	let mut keybindings = k;
	for ops in keybindings.values_mut() {
		ops.add_tx(event_tx);
	}
	keybindings
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
	.map(|(key, ops)| {
		(
			key.to_string(),
			ops.into_iter().map(|op| op.to_string()).collect(),
		)
	})
	.collect()
}
