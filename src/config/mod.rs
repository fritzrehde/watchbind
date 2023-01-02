mod keybindings;
mod style;

pub use keybindings::{Key, Operations};
pub use style::Styles;

use crate::command::Command;
use anyhow::{bail, Result};
use clap::Parser;
use keybindings::{Keybindings, KeybindingsRaw};
use serde::Deserialize;
use std::{collections::HashMap, fs::read_to_string, time::Duration};

// TODO: find better solution than to make all fields public
pub struct Config {
	pub command: Command,
	pub watch_rate: Duration,
	pub styles: Styles,
	pub keybindings: Keybindings,
}

impl Config {
	pub fn parse() -> Result<Self> {
		let cli = ClapConfig::parse();
		let config_file = cli.config_file.clone();
		let cli: OptionalConfig = cli.into();
		let config = match &config_file {
			// TODO: parse toml directly into optional
			Some(path) => cli.merge(TomlConfig::parse(path)?.into()),
			None => cli,
		};
		config.try_into()
	}
}

#[derive(Parser)]
#[clap(version, about)]
pub struct ClapConfig {
	/// Command to execute periodically
	#[arg(trailing_var_arg(true))]
	command: Option<Vec<String>>,

	/// TOML config file path
	#[arg(short, long, value_name = "FILE")]
	config_file: Option<String>,

	/// Seconds to wait between updates, 0 only executes once
	#[arg(short, long, value_name = "SECS")]
	interval: Option<f64>,

	/// Foreground color of all lines except cursor
	#[arg(long, value_name = "COLOR")]
	fg: Option<String>,

	/// Background color of all lines except cursor
	#[arg(long, value_name = "COLOR")]
	bg: Option<String>,

	/// Foreground color of cursor
	#[arg(long = "fg+", value_name = "COLOR")]
	fg_cursor: Option<String>,

	/// Background color of cursor
	#[arg(long = "bg+", value_name = "COLOR")]
	bg_cursor: Option<String>,

	/// Color of selected line marker
	#[arg(long = "bg-", value_name = "COLOR")]
	bg_selected: Option<String>,

	/// Text on all lines except cursor are bold
	#[arg(long)]
	bold: bool,

	/// Text on cursor's line is bold
	#[arg(long = "bold+")]
	bold_cursor: bool,

	// TODO: use KeybindingsRaw once clap supports parsing into HashMap
	/// Comma-seperated list of keybindings in the format KEY:OP[+OP]*[,KEY:OP[+OP]*]*
	#[arg(short = 'b', long = "bind", value_name = "KEYBINDINGS", value_delimiter = ',', value_parser = keybindings::parse_str)]
	keybindings: Option<Vec<(String, Vec<String>)>>,
}

impl TryFrom<OptionalConfig> for Config {
	type Error = anyhow::Error;
	fn try_from(opt: OptionalConfig) -> Result<Self, Self::Error> {
		let default = DefaultConfig::new();
		Ok(Self {
			command: match opt.command {
				Some(command) => Command::new(command),
				None => bail!("A command must be provided via command line or config file"),
			},
			watch_rate: Duration::from_secs_f64(opt.interval.unwrap_or(default.interval)),
			styles: Styles::parse(
				opt.fg.or(default.fg),
				opt.bg.or(default.bg),
				opt.fg_cursor.or(default.fg_cursor),
				opt.bg_cursor.or(default.bg_cursor),
				opt.bg_selected.or(default.bg_selected),
				opt.bold.unwrap_or(default.bold),
				opt.bold_cursor.unwrap_or(default.bold_cursor),
			)?,
			keybindings: keybindings::merge_raw(
				opt.keybindings,
				default.keybindings,
			).try_into()?,
		})
	}
}

#[derive(Deserialize)]
pub struct TomlConfig {
	command: Option<String>,
	interval: Option<f64>,
	fg: Option<String>,
	bg: Option<String>,
	#[serde(rename = "fg+")]
	fg_cursor: Option<String>,
	#[serde(rename = "bg+")]
	bg_cursor: Option<String>,
	#[serde(rename = "bg-")]
	bg_selected: Option<String>,
	bold: Option<bool>,
	#[serde(rename = "bold+")]
	bold_cursor: Option<bool>,
	keybindings: Option<KeybindingsRaw>,
}

impl TomlConfig {
	fn parse(config_file: &str) -> Result<Self> {
		// TODO: add to anyhow error that error came from parsing file in here
		let config = toml::from_str(&read_to_string(config_file)?)?;
		Ok(config)
	}
}

pub struct OptionalConfig {
	command: Option<String>,
	interval: Option<f64>,
	fg: Option<String>,
	bg: Option<String>,
	fg_cursor: Option<String>,
	bg_cursor: Option<String>,
	bg_selected: Option<String>,
	bold: Option<bool>,
	bold_cursor: Option<bool>,
	keybindings: KeybindingsRaw,
}

impl OptionalConfig {
	// self is favored
	fn merge(self, other: Self) -> Self {
		Self {
			command: self.command.or(other.command),
			interval: self.interval.or(other.interval),
			fg: self.fg.or(other.fg),
			bg: self.bg.or(other.bg),
			fg_cursor: self.fg_cursor.or(other.fg_cursor),
			bg_cursor: self.bg_cursor.or(other.bg_cursor),
			bg_selected: self.bg_selected.or(other.bg_selected),
			bold: self.bold.or(other.bold),
			bold_cursor: self.bold_cursor.or(other.bold_cursor),
			// TODO: self.keybindings.merge_with(other.keybindings)
			keybindings: keybindings::merge_raw(self.keybindings, other.keybindings),
		}
	}
}

impl From<ClapConfig> for OptionalConfig {
	fn from(clap: ClapConfig) -> Self {
		Self {
			command: clap.command.map_or(None, |s| Some(s.join(" "))),
			interval: clap.interval,
			fg: clap.fg,
			bg: clap.bg,
			fg_cursor: clap.fg_cursor,
			bg_cursor: clap.bg_cursor,
			bg_selected: clap.bg_selected,
			bold: clap.bold.then_some(clap.bold),
			bold_cursor: clap.bold_cursor.then_some(clap.bold_cursor),
			// TODO: simplify
			keybindings: clap
				.keybindings
				.map_or_else(|| HashMap::new(), |s| s.into_iter().collect()),
		}
	}
}

// TODO: optimize away
impl From<TomlConfig> for OptionalConfig {
	fn from(file: TomlConfig) -> Self {
		Self {
			command: file.command,
			interval: file.interval,
			fg: file.fg,
			bg: file.bg,
			fg_cursor: file.fg_cursor,
			bg_cursor: file.bg_cursor,
			bg_selected: file.bg_selected,
			bold: file.bold,
			bold_cursor: file.bold_cursor,
			keybindings: file.keybindings.unwrap_or(HashMap::new()),
		}
	}
}

struct DefaultConfig {
	interval: f64,
	fg: Option<String>,
	bg: Option<String>,
	fg_cursor: Option<String>,
	bg_cursor: Option<String>,
	bg_selected: Option<String>,
	bold: bool,
	bold_cursor: bool,
	keybindings: KeybindingsRaw,
}

// TODO: replace with inline toml config file with toml::toml! macro
// TODO: remove ConfigRaw completely
impl DefaultConfig {
	fn new() -> Self {
		Self {
			interval: 5.0,
			fg: None,
			bg: None,
			fg_cursor: Some("black".to_owned()),
			bg_cursor: Some("blue".to_owned()),
			bg_selected: Some("magenta".to_owned()),
			bold: false,
			bold_cursor: true,
			keybindings: keybindings::default_raw(),
		}
	}
}
