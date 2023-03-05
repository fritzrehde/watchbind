mod keybindings;
mod style;

pub use keybindings::{Key, Keybindings, Operations};
pub use style::Styles;

use crate::command::Command;
use anyhow::{bail, Context, Result};
use clap::Parser;
use indoc::indoc;
use keybindings::StringKeybindings;
use serde::Deserialize;
use std::{fs::read_to_string, time::Duration};

pub struct Config {
	pub command: Command,
	pub watch_rate: Duration,
	pub styles: Styles,
	pub keybindings: Keybindings,
	// TODO: turn into own type
	pub field_separator: Option<String>,
}

impl Config {
	pub fn parse() -> Result<Self> {
		let cli = ClapConfig::parse();
		let config_file = cli.config_file.clone();
		let cli: TomlConfig = cli.into();
		let config = match &config_file {
			Some(path) => cli.merge(TomlConfig::parse(path)?),
			None => cli,
		};
		config.try_into()
	}
}

impl TryFrom<TomlConfig> for Config {
	type Error = anyhow::Error;
	fn try_from(toml: TomlConfig) -> Result<Self, Self::Error> {
		let default = TomlConfig::default();
		Ok(Self {
			command: match toml.command {
				Some(command) => Command::new(command),
				None => bail!("A command must be provided via command line or config file"),
			},
			watch_rate: Duration::from_secs_f64(toml.interval.or(default.interval).expect("default")),
			styles: Styles::parse(
				toml.fg.or(default.fg),
				toml.bg.or(default.bg),
				toml.fg_cursor.or(default.fg_cursor),
				toml.bg_cursor.or(default.bg_cursor),
				toml.bg_selected.or(default.bg_selected),
				toml.bold.or(default.bold),
				toml.bold_cursor.or(default.bold_cursor),
			)?,
			keybindings: StringKeybindings::merge(toml.keybindings, default.keybindings)
				.expect("default")
				.try_into()?,
			field_separator: toml.field_separator,
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
	#[serde(rename = "field-separator")]
	field_separator: Option<String>,
	keybindings: Option<StringKeybindings>,
}

impl TomlConfig {
	fn parse(config_file: &str) -> Result<Self> {
		// TODO: add to anyhow error that error came from parsing file in here
		let config = toml::from_str(
			&read_to_string(config_file)
				.with_context(|| format!("Failed to read configuration from {}", config_file))?,
		)
		.with_context(|| format!("Failed to parse toml from {}", config_file))?;
		Ok(config)
	}

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
			field_separator: self.field_separator.or(other.field_separator),
			keybindings: StringKeybindings::merge(self.keybindings, other.keybindings),
		}
	}
}

impl From<ClapConfig> for TomlConfig {
	fn from(clap: ClapConfig) -> Self {
		Self {
			command: clap.command.map(|s| s.join(" ")),
			interval: clap.interval,
			fg: clap.fg,
			bg: clap.bg,
			fg_cursor: clap.fg_cursor,
			bg_cursor: clap.bg_cursor,
			bg_selected: clap.bg_selected,
			bold: clap.bold,
			bold_cursor: clap.bold_cursor,
			field_separator: clap.field_separator,
			keybindings: clap.keybindings.map(|vec| vec.into()),
		}
	}
}

impl Default for TomlConfig {
	fn default() -> Self {
		let toml = indoc! {r#"
			"interval" = 5.0
			"fg+" = "black"
			"bg+" = "blue"
			"bg-" = "magenta"
			"bold" = false
			"bold+" = true

			[keybindings]
			"ctrl+c" = [ "exit" ]
			"q" = [ "exit" ]
			"r" = [ "reload" ]
			"space" = [ "select-toggle", "down" ]
			"v" = [ "select-toggle" ]
			"esc" = [ "unselect-all" ]
			"down" = [ "down" ]
			"up" = [ "up" ]
			"j" = [ "down" ]
			"k" = [ "up" ]
			"g" = [ "first" ]
			"G" = [ "last" ]
		"#};
		toml::from_str(toml).expect("correct default toml config file")
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
	#[arg(long, value_name = "BOOL")]
	bold: Option<bool>,

	/// Text on cursor's line is bold
	#[arg(long = "bold+", value_name = "BOOL")]
	bold_cursor: Option<bool>,

	/// Field separator
	#[arg(short = 's', long = "field-separator", value_name = "STRING")]
	field_separator: Option<String>,

	// TODO: replace with StringKeybindings once clap supports parsing into HashMap
	// TODO: known clap bug: replace with ClapKeybindings once supported
	/// Comma-separated list of keybindings in the format KEY:OP[+OP]*[,KEY:OP[+OP]*]*
	#[arg(short = 'b', long = "bind", value_name = "KEYBINDINGS", value_delimiter = ',', value_parser = keybindings::parse_str)]
	keybindings: Option<Vec<(String, Vec<String>)>>,
}
