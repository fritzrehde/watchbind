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
    pub header_lines: usize,
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
            watch_rate: Duration::from_secs_f64(
                toml.interval.or(default.interval).expect("default"),
            ),
            styles: Styles::parse(
                toml.fg.or(default.fg),
                toml.bg.or(default.bg),
                toml.bold.or(default.bold),
                toml.cursor_fg.or(default.cursor_fg),
                toml.cursor_bg.or(default.cursor_bg),
                toml.cursor_bold.or(default.cursor_bold),
                toml.header_fg.or(default.header_fg),
                toml.header_bg.or(default.header_bg),
                toml.header_bold.or(default.header_bold),
                toml.selected_bg.or(default.selected_bg),
            )?,
            keybindings: StringKeybindings::merge(toml.keybindings, default.keybindings)
                .expect("default")
                .try_into()?,
            header_lines: toml.header_lines.unwrap_or(0),
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
    bold: Option<bool>,

    #[serde(rename = "cursor-fg")]
    cursor_fg: Option<String>,

    #[serde(rename = "cursor-bg")]
    cursor_bg: Option<String>,

    #[serde(rename = "cursor-bold")]
    cursor_bold: Option<bool>,

    #[serde(rename = "header-fg")]
    header_fg: Option<String>,

    #[serde(rename = "header-bg")]
    header_bg: Option<String>,

    #[serde(rename = "header-bold")]
    header_bold: Option<bool>,

    #[serde(rename = "selected-bg")]
    selected_bg: Option<String>,

    #[serde(rename = "header-lines")]
    header_lines: Option<usize>,

    #[serde(rename = "field-seperator")]
    field_separator: Option<String>,

    keybindings: Option<StringKeybindings>,
}

impl TomlConfig {
    fn parse(config_file: &str) -> Result<Self> {
        let config = toml::from_str(
            &read_to_string(config_file)
                .with_context(|| format!("Failed to read configuration from {config_file}"))?,
        )
        .with_context(|| format!("Failed to parse toml from {config_file}"))?;
        Ok(config)
    }

    // self is favored
    fn merge(self, other: Self) -> Self {
        Self {
            command: self.command.or(other.command),
            interval: self.interval.or(other.interval),
            fg: self.fg.or(other.fg),
            bg: self.bg.or(other.bg),
            bold: self.bold.or(other.bold),
            cursor_fg: self.cursor_fg.or(other.cursor_fg),
            cursor_bg: self.cursor_bg.or(other.cursor_bg),
            cursor_bold: self.cursor_bold.or(other.cursor_bold),
            header_fg: self.header_fg.or(other.header_fg),
            header_bg: self.header_bg.or(other.header_bg),
            header_bold: self.header_bold.or(other.header_bold),
            selected_bg: self.selected_bg.or(other.selected_bg),
            header_lines: self.header_lines.or(other.header_lines),
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
            bold: clap.bold,
            cursor_fg: clap.cursor_fg,
            cursor_bg: clap.cursor_bg,
            cursor_bold: clap.cursor_bold,
            header_fg: clap.header_fg,
            header_bg: clap.header_bg,
            header_bold: clap.header_bold,
            selected_bg: clap.selected_bg,
            header_lines: clap.header_lines,
            field_separator: clap.field_separator,
            keybindings: clap.keybindings.map(|vec| vec.into()),
        }
    }
}

impl Default for TomlConfig {
    fn default() -> Self {
        let toml = indoc! {r#"
			"interval" = 5.0
			"bold" = false
			"cursor-fg" = "black"
			"cursor-bg" = "blue"
			"cursor-bold" = true
			"header-fg" = "blue"
			"selected-bg" = "magenta"

			[keybindings]
			"ctrl+c" = [ "exit" ]
			"q" = [ "exit" ]
			"r" = [ "reload" ]
			"?" = [ "help" ]
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
        toml::from_str(toml).expect("default toml config file should be correct")
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

    /// Text on all lines except the cursor's line are bold
    #[arg(long, value_name = "BOOL")]
    bold: Option<bool>,

    /// Foreground color of cursor
    #[arg(long = "cursor-fg", value_name = "COLOR")]
    cursor_fg: Option<String>,

    /// Background color of cursor
    #[arg(long = "cursor-bg", value_name = "COLOR")]
    cursor_bg: Option<String>,

    /// Text on cursor's line is bold
    #[arg(long = "cursor-bold", value_name = "BOOL")]
    cursor_bold: Option<bool>,

    /// Foreground color of header lines
    #[arg(long = "header-fg", value_name = "COLOR")]
    header_fg: Option<String>,

    /// Background color of header lines
    #[arg(long = "header-bg", value_name = "COLOR")]
    header_bg: Option<String>,

    /// Text on header line is bold
    #[arg(long = "header-bold", value_name = "BOOL")]
    header_bold: Option<bool>,

    /// Background color of selected line marker
    #[arg(long = "selected-bg", value_name = "COLOR")]
    selected_bg: Option<String>,

    /// The first N lines of the input are treated as a sticky header
    #[arg(long = "header-lines", value_name = "N")]
    header_lines: Option<usize>,

    /// Field separator
    #[arg(short = 's', long = "field-separator", value_name = "STRING")]
    field_separator: Option<String>,

    // /// Print only these specified fields to the UI
    // #[arg(short = 'f', long = "fields", value_name = "STRING")]
    // fields: Option<String>,

    // TODO: replace with StringKeybindings once clap supports parsing into HashMap
    // TODO: known clap bug: replace with ClapKeybindings once supported
    /// Comma-separated list of keybindings in the format KEY:OP[+OP]*[,KEY:OP[+OP]*]*
    #[arg(short = 'b', long = "bind", value_name = "KEYBINDINGS", value_delimiter = ',', value_parser = keybindings::parse_str)]
    keybindings: Option<Vec<(String, Vec<String>)>>,
}
