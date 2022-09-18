use std::time::Duration;
use serde::Deserialize;
use crate::{style, cli, toml};

// mod style;
// mod cli;
// mod toml;

// TODO: find better solution than to make all fields public
pub struct Config {
	pub command: String,
	pub watch_rate: Duration,
	pub tick_rate: Duration,
	pub styles: style::Styles,
	pub keybindings: Vec<Binding>,
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
	keybindings: Vec<Binding>,
}

#[derive(Deserialize)]
pub struct ConfigRawOptional {
	command: Option<String>,
	interval: Option<f64>,
	fg: Option<String>,
	bg: Option<String>,
	fg_plus: Option<String>,
	bg_plus: Option<String>,
	bold: Option<bool>,
	bold_plus: Option<bool>,
	keybindings: Option<Vec<Binding>>,
}

#[derive(Deserialize)]
struct Binding {
	key: String,
	command: String,
}

fn default_config() -> ConfigRaw {
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
		keybindings: Vec::new(),
	}
}

pub fn parse_config() -> Config {
	let (clap_config, config_file) = cli::parse_clap();
	match config_file {
		Some(path) => {
			let toml_config = toml::parse_toml(&path);
			merge_default(merge_opt(clap_config, toml_config))
		},
		None => merge_default(clap_config)
	}
}

// TODO: remove repitition with macro

// Merge a ConfigRawOptional config with the default config
fn merge_default(opt: ConfigRawOptional) -> Config {
	let default: ConfigRaw = default_config();
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
		interval: opt1.interval.or(opt2.interval),
		fg: opt1.fg.or(opt2.fg.or(default.fg)),
		bg: opt1.bg.or(opt2.bg.or(default.bg)),
		fg_plus: opt1.fg_plus.or(opt2.fg_plus),
		bg_plus: opt1.bg_plus.or(opt2.bg_plus),
		bold: opt1.bold.or(opt2.bold),
		bold_plus: opt1.bold_plus.or(opt2.bold_plus),
		keybindings: opt1.keybindings.or(opt2.keybindings),
	}
}
