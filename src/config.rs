use std::time::Duration;
use serde::Deserialize;
use clap::{Parser};
use crate::{style, toml, keys::{self, Keybindings, KeybindingsRaw}};

// TODO: find better solution than to make all fields public
pub struct Config {
	pub command: String,
	pub watch_rate: Duration,
	pub tick_rate: Duration,
	pub styles: style::Styles,
	pub keybindings: Keybindings,
}

struct ConfigRaw {
	command: String,
	interval: f64,
	tick_rate: u64,
	fg: Option<String>,
	bg: Option<String>,
	fg_plus: Option<String>,
	bg_plus: Option<String>,
	bold: bool,
	bold_plus: bool,
	keybindings: KeybindingsRaw,
}

#[derive(Deserialize, Parser)]
#[clap(about)]
pub struct ConfigRawOptional {
	/// Command to execute periodically
	command: Option<String>,

	/// YAML config file path
	#[clap(short, long, value_name = "FILE")]
	config: Option<String>,

	/// Seconds to wait between updates, 0 only executes once
	#[clap(short, long, value_name = "SECS")]
	interval: Option<f64>,

	/// Foreground color of unselected lines
	#[clap(long, value_name = "COLOR")]
	fg: Option<String>,

	/// Background color of unselected lines
	#[clap(long, value_name = "COLOR")]
	bg: Option<String>,

	/// Foreground color of selected lines
	#[clap(long, value_name = "COLOR")]
	fg_plus: Option<String>,

	/// Foreground color of selected lines
	#[clap(long, value_name = "COLOR")]
	bg_plus: Option<String>,

	/// All lines except selected line are bold
	#[clap(long)]
	bold: Option<bool>,

	/// Selected line is bold
	#[clap(long)]
	bold_plus: Option<bool>,

	/// Comma-seperated list of keybindings in the format KEY:CMD[,KEY:CMD]*
	#[clap(short, long, value_name = "KEYBINDINGS")]
	keybindings: Option<KeybindingsRaw>,
}

pub fn parse_config() -> Config {
	let cli = ConfigRawOptional::parse();
	// match cli.config.as_deref() {
	match cli.config {
		Some(path) => {
			let toml_config = toml::parse_toml(&path);
			merge_default(merge_opt(cli, toml_config))
		},
		None => merge_default(cli)
	}
}

impl Default for ConfigRaw {
	fn default() -> ConfigRaw {
		ConfigRaw {
			command: "ls".to_string(),
			interval: 5.0,
			tick_rate: 250,
			fg: None,
			bg: None,
			fg_plus: None,
			bg_plus: None,
			bold: false,
			bold_plus: false,
			keybindings: keys::default_keybindingsraw(),
		}
	}
}

// TODO: remove repitition with macro

// Merge a ConfigRawOptional config with the default config
fn merge_default(opt: ConfigRawOptional) -> Config {
	let default: ConfigRaw = ConfigRaw::default();
	Config {
		// TODO: handle missing command, no default
		command: opt.command.expect("Arg command must exist"),
		watch_rate: Duration::from_secs_f64(opt.interval.unwrap_or(default.interval)),
		tick_rate: Duration::from_millis(default.tick_rate),
		styles: style::parse_style(
			opt.fg.or(default.fg),
			opt.bg.or(default.bg),
			opt.fg_plus.or(default.fg_plus),
			opt.bg_plus.or(default.bg_plus),
			opt.bold.unwrap_or(default.bold),
			opt.bold_plus.unwrap_or(default.bold_plus)),
		keybindings: opt.keybindings.unwrap_or(default.keybindings),
	}
}

// Merge two ConfigRawOptional configs
fn merge_opt(opt1: ConfigRawOptional, opt2: ConfigRawOptional) -> ConfigRawOptional {
	ConfigRawOptional {
		command: opt1.command.or(opt2.command),
		config: opt1.config.or(opt2.config),
		interval: opt1.interval.or(opt2.interval),
		fg: opt1.fg.or(opt2.fg),
		bg: opt1.bg.or(opt2.bg),
		fg_plus: opt1.fg_plus.or(opt2.fg_plus),
		bg_plus: opt1.bg_plus.or(opt2.bg_plus),
		bold: opt1.bold.or(opt2.bold),
		bold_plus: opt1.bold_plus.or(opt2.bold_plus),
		keybindings: opt1.keybindings.or(opt2.keybindings),
	}
}
