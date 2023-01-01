mod key;
mod operations;

pub use key::Key;
pub use operations::{Operation, Operations};
pub type KeybindingsRaw = HashMap<String, Vec<String>>;

use crate::ui::Event;
use anyhow::{bail, Result};
use std::{collections::HashMap, sync::mpsc::Sender};

pub struct Keybindings {
	keybindings: HashMap<Key, Operations>,
}

impl Keybindings {
	pub fn add_event_tx(&mut self, event_tx: &Sender<Event>) {
		for ops in self.keybindings.values_mut() {
			ops.add_tx(event_tx);
		}
	}

	pub fn get_operations(&self, key: &Key) -> Option<&Operations> {
		self.keybindings.get(key)
	}
}

impl TryFrom<KeybindingsRaw> for Keybindings {
	type Error = anyhow::Error;
	fn try_from(value: KeybindingsRaw) -> Result<Self, Self::Error> {
		let keybindings = value
			.into_iter()
			.map(|(key, ops)| Ok((key.parse()?, Operations::from_vec(ops)?)))
			.collect::<Result<_>>()?;
		Ok(Self { keybindings })
	}
}

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

// new and old have same key => keep new value
pub fn merge_raw(new: KeybindingsRaw, old: KeybindingsRaw) -> KeybindingsRaw {
	// TODO: borrow old as mutable and avoid clone
	let mut merged = old.clone();
	merged.extend(new);
	merged
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
