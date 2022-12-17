use crate::{
	keybindings::{self, Keybindings, KeybindingsRaw},
	style,
};
use anyhow::{bail, Result};
use clap::Parser;
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};

// TODO: find better solution than to make all fields public
pub struct Config {
	pub command: String,
	pub watch_rate: Duration,
	pub tick_rate: Duration,
	pub styles: style::Styles,
	pub keybindings: Keybindings,
}

impl Config {
	pub fn parse() -> Result<Config> {
		let cli = ConfigRawArgs::parse();
		let config_file = cli.config_file.clone();
		let args = args2optional(cli);
		merge_default(match &config_file {
			// TODO: parse toml directly into optional
			Some(path) => merge_opt(args, file2optional(parse_toml(path)?)),
			None => args,
		})
	}
}

struct ConfigRaw {
	interval: f64,
	tick_rate: u64,
	fg: Option<String>,
	bg: Option<String>,
	fg_cursor: Option<String>,
	bg_cursor: Option<String>,
	bg_selected: Option<String>,
	bold: bool,
	bold_cursor: bool,
	keybindings: KeybindingsRaw,
}

#[derive(Parser)]
#[clap(version, about)]
pub struct ConfigRawArgs {
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

	/// Background color of cursor
	#[arg(long = "bg-", value_name = "COLOR")]
	bg_selected: Option<String>,

	/// Text on all lines except cursor are bold
	#[arg(long)]
	bold: bool,

	/// Text on cursor's line is bold
	#[arg(long = "bold+")]
	bold_cursor: bool,

	/// Comma-seperated list of keybindings in the format KEY:OP[+OP]*[,KEY:OP[+OP]*]*
	#[arg(short = 'b', long = "bind", value_name = "KEYBINDINGS", value_delimiter = ',', value_parser = keybindings::parse_str)]
	keybindings: Option<Vec<(String, Vec<String>)>>,
}

#[derive(Deserialize)]
pub struct ConfigRawFile {
	command: Option<String>,
	interval: Option<f64>,
	fg: Option<String>,
	bg: Option<String>,
	fg_cursor: Option<String>,
	bg_cursor: Option<String>,
	bg_selected: Option<String>,
	bold: Option<bool>,
	bold_cursor: Option<bool>,
	keybindings: Option<KeybindingsRaw>,
}

pub struct ConfigRawOptional {
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

fn parse_toml(config_file: &str) -> Result<ConfigRawFile> {
	// TODO: add to anyhow error that error came from parsing file in here
	let config = config::Config::builder()
		.add_source(config::File::with_name(config_file))
		.build()?
		.try_deserialize()?;
	Ok(config)
}

// Merge a ConfigRawOptional config with the default config
fn merge_default(opt: ConfigRawOptional) -> Result<Config> {
	let default: ConfigRaw = ConfigRaw::default();
	Ok(Config {
		command: match opt.command {
			Some(command) => command,
			None => bail!("A command must be provided via command line or config file"),
		},
		watch_rate: Duration::from_secs_f64(opt.interval.unwrap_or(default.interval)),
		tick_rate: Duration::from_millis(default.tick_rate),
		styles: style::parse_style(
			opt.fg.or(default.fg),
			opt.bg.or(default.bg),
			opt.fg_cursor.or(default.fg_cursor),
			opt.bg_cursor.or(default.bg_cursor),
			opt.bg_selected.or(default.bg_selected),
			opt.bold.unwrap_or(default.bold),
			opt.bold_cursor.unwrap_or(default.bold_cursor),
		)?,
		keybindings: keybindings::parse_raw(keybindings::merge_raw(
			opt.keybindings,
			default.keybindings,
		))?,
	})
}

// Merge two ConfigRawOptional configs, opt1 is favoured
fn merge_opt(opt1: ConfigRawOptional, opt2: ConfigRawOptional) -> ConfigRawOptional {
	ConfigRawOptional {
		command: opt1.command.or(opt2.command),
		interval: opt1.interval.or(opt2.interval),
		fg: opt1.fg.or(opt2.fg),
		bg: opt1.bg.or(opt2.bg),
		fg_cursor: opt1.fg_cursor.or(opt2.fg_cursor),
		bg_cursor: opt1.bg_cursor.or(opt2.bg_cursor),
		bg_selected: opt1.bg_selected.or(opt2.bg_selected),
		bold: opt1.bold.or(opt2.bold),
		bold_cursor: opt1.bold_cursor.or(opt2.bold_cursor),
		keybindings: keybindings::merge_raw(opt1.keybindings, opt2.keybindings),
	}
}

fn args2optional(args: ConfigRawArgs) -> ConfigRawOptional {
	ConfigRawOptional {
		command: args.command.map_or(None, |s| Some(s.join(" "))),
		interval: args.interval,
		fg: args.fg,
		bg: args.bg,
		fg_cursor: args.fg_cursor,
		bg_cursor: args.bg_cursor,
		bg_selected: args.bg_selected,
		bold: args.bold.then_some(args.bold),
		bold_cursor: args.bold_cursor.then_some(args.bold_cursor),
		// TODO: simplify
		keybindings: args
			.keybindings
			.map_or_else(|| HashMap::new(), |s| s.into_iter().collect()),
	}
}

// TODO: optimize away
fn file2optional(file: ConfigRawFile) -> ConfigRawOptional {
	ConfigRawOptional {
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

impl Default for ConfigRaw {
	fn default() -> ConfigRaw {
		ConfigRaw {
			interval: 5.0,
			tick_rate: 250,
			fg: None,
			bg: None,
			fg_cursor: Some("black".to_string()),
			bg_cursor: Some("blue".to_string()),
			bg_selected: Some("magenta".to_string()),
			bold: false,
			bold_cursor: true,
			keybindings: keybindings::default_raw(),
		}
	}
}
