use std::{time::Duration, collections::HashMap};
use std::io::{Error, ErrorKind};
use serde::Deserialize;
use clap::{Parser};
use crate::{style, toml, keys::{self, Keybindings, KeybindingsRaw}};

// TODO: find better solution than to make all fields public
#[derive(Debug)]
pub struct Config {
	pub command: String,
	pub watch_rate: Duration,
	pub tick_rate: Duration,
	pub styles: style::Styles,
	pub keybindings: Keybindings,
}

struct ConfigRaw {
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

#[derive(Parser)]
#[clap(version, about)]
pub struct ConfigRawArgs {
	/// Command to execute periodically
	command: Option<String>,
	/// YAML config file path
	#[clap(short, long, value_name = "FILE")]
	config_file: Option<String>,
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
	#[clap(long = "fg+", value_name = "COLOR")]
	fg_plus: Option<String>,
	/// Foreground color of selected lines
	#[clap(long = "bg+", value_name = "COLOR")]
	bg_plus: Option<String>,
	/// All lines except selected line are bold
	#[clap(long)]
	bold: bool,
	/// Selected line is bold
	#[clap(long = "bold+")]
	bold_plus: bool,
	/// Comma-seperated list of keybindings in the format KEY:CMD[,KEY:CMD]*
	#[clap(short = 'b', long = "bind", value_name = "KEYBINDINGS")]
	keybindings: Option<String>,
}

#[derive(Deserialize)]
pub struct ConfigRawFile {
	command: Option<String>,
	interval: Option<f64>,
	fg: Option<String>,
	bg: Option<String>,
	fg_plus: Option<String>,
	bg_plus: Option<String>,
	bold: Option<bool>,
	bold_plus: Option<bool>,
	keybindings: Option<KeybindingsRaw>,
}

pub struct ConfigRawOptional {
	command: Option<String>,
	interval: Option<f64>,
	fg: Option<String>,
	bg: Option<String>,
	fg_plus: Option<String>,
	bg_plus: Option<String>,
	bold: Option<bool>,
	bold_plus: Option<bool>,
	keybindings: KeybindingsRaw,
}

pub fn parse_config() -> Result<Config, Error> {
	let cli = ConfigRawArgs::parse();
	let config_file = cli.config_file.clone();
	let args = args2optional(cli);
	match &config_file {
		Some(path) => {
			// TODO: can go wrong
			let file = file2optional(toml::parse_toml(path));
			merge_default(merge_opt(args, file))
		},
		None => merge_default(args)
	}
}

// Merge a ConfigRawOptional config with the default config
fn merge_default(opt: ConfigRawOptional) -> Result<Config, Error> {
	let default: ConfigRaw = ConfigRaw::default();
	Ok(
		Config {
			// TODO: handle missing command, no default
			command: opt.command
				.ok_or(Error::new(ErrorKind::Other, "Command must be provided via command line or config file"))?,
			// clap::Cli::command().error(clap::error::ErrorKind::MissingRequiredArgument, "Command must be provided via command line or config file").exit()
			watch_rate: Duration::from_secs_f64(opt.interval.unwrap_or(default.interval)),
			tick_rate: Duration::from_millis(default.tick_rate),
			styles: style::parse_style(
				opt.fg.or(default.fg),
				opt.bg.or(default.bg),
				opt.fg_plus.or(default.fg_plus),
				opt.bg_plus.or(default.bg_plus),
				opt.bold.unwrap_or(default.bold),
				opt.bold_plus.unwrap_or(default.bold_plus)),
			keybindings: keys::parse_raw(keys::merge_raw(opt.keybindings, default.keybindings)),
		}
	)
}

// Merge two ConfigRawOptional configs, opt1 is favoured
fn merge_opt(opt1: ConfigRawOptional, opt2: ConfigRawOptional) -> ConfigRawOptional {
	ConfigRawOptional {
		command: opt1.command.or(opt2.command),
		interval: opt1.interval.or(opt2.interval),
		fg: opt1.fg.or(opt2.fg),
		bg: opt1.bg.or(opt2.bg),
		fg_plus: opt1.fg_plus.or(opt2.fg_plus),
		bg_plus: opt1.bg_plus.or(opt2.bg_plus),
		bold: opt1.bold.or(opt2.bold),
		bold_plus: opt1.bold_plus.or(opt2.bold_plus),
		keybindings: keys::merge_raw(opt1.keybindings, opt2.keybindings),
	}
}

fn args2optional(args: ConfigRawArgs) -> ConfigRawOptional {
	ConfigRawOptional {
		command: args.command,
		interval: args.interval,
		fg: args.fg,
		bg: args.bg,
		fg_plus: args.fg_plus,
		bg_plus: args.bg_plus,
		bold: args.bold.then_some(args.bold),
		bold_plus: args.bold_plus.then_some(args.bold_plus),
		keybindings: args.keybindings.map_or(HashMap::new(), |s| keys::parse_str(s)),
	}
}

fn file2optional(file: ConfigRawFile) -> ConfigRawOptional {
	ConfigRawOptional {
		command: file.command,
		interval: file.interval,
		fg: file.fg,
		bg: file.bg,
		fg_plus: file.fg_plus,
		bg_plus: file.bg_plus,
		bold: file.bold,
		bold_plus: file.bold_plus,
		keybindings: file.keybindings.unwrap_or(HashMap::new()),
	}
}

impl Default for ConfigRaw {
	fn default() -> ConfigRaw {
		ConfigRaw {
			interval: 5.0,
			tick_rate: 250,
			fg: None,
			bg: None,
			fg_plus: Some("black".to_string()),
			bg_plus: Some("blue".to_string()),
			bold: false,
			bold_plus: true,
			keybindings: keys::default_keybindingsraw(),
		}
	}
}
