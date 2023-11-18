mod fields;
mod keybindings;
mod possible_enum_values;
mod style;

use crate::config::keybindings::{KeyCode, KeyModifier};
use crate::config::possible_enum_values::PossibleEnumValues;
use crate::config::style::PrettyColor;

use self::fields::{FieldSelections, FieldSeparator};
use self::keybindings::{KeybindingsParsed, StringKeybindings};
use self::style::{Boldness, Color, Style};
use anyhow::{bail, Context, Result};
use clap::Parser;
use indoc::indoc;
use owo_colors::OwoColorize;
use serde::Deserialize;
use std::{fs::read_to_string, path::PathBuf, time::Duration};
use tabled::builder::Builder;
use tabled::settings::peaker::PriorityMax;
use tabled::settings::{Margin, Padding, Style as TableStyle, Width};
use terminal_size::{terminal_size, Width as TerminalWidth};

pub use fields::{Fields, TableFormatter};
pub use keybindings::{
    KeyEvent, Keybindings, Operation, OperationParsed, Operations, OperationsParsed,
};
pub use style::Styles;

// TODO: don't have public members

pub struct Config {
    pub log_file: Option<PathBuf>,
    pub watched_command: String,
    pub watch_rate: Duration,
    pub styles: Styles,
    pub keybindings_parsed: KeybindingsParsed,
    pub header_lines: usize,
    pub fields: Fields,
    pub initial_env_ops: OperationsParsed,
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

        let non_cursor_style = Style::new(
            toml.non_cursor_non_header_fg
                .or(default.non_cursor_non_header_fg),
            toml.non_cursor_non_header_bg
                .or(default.non_cursor_non_header_bg),
            toml.non_cursor_non_header_boldness
                .or(default.non_cursor_non_header_boldness),
        );
        let cursor_style = Style::new(
            toml.cursor_fg.or(default.cursor_fg),
            toml.cursor_bg.or(default.cursor_bg),
            toml.cursor_boldness.or(default.cursor_boldness),
        );
        let header_style = Style::new(
            toml.header_fg.or(default.header_fg),
            toml.header_bg.or(default.header_bg),
            toml.header_boldness.or(default.header_boldness),
        );
        let selected_style = Style::new(
            Color::Unspecified,
            toml.selected_bg.or(default.selected_bg),
            Boldness::Unspecified,
        );
        let styles = Styles::new(non_cursor_style, cursor_style, header_style, selected_style);

        Ok(Self {
            log_file: toml.log_file,
            initial_env_ops: toml.initial_env_vars.unwrap_or_default().try_into()?,
            watched_command: match toml.watched_command {
                Some(command) => command,
                None => bail!("A command must be provided via command line or config file"),
            },
            watch_rate: Duration::from_secs_f64(
                toml.interval.or(default.interval).expect("default"),
            ),
            styles,
            keybindings_parsed: StringKeybindings::merge(toml.keybindings, default.keybindings)
                .expect("default")
                .try_into()?,
            header_lines: toml.header_lines.unwrap_or(0),
            fields: Fields::try_new(toml.field_separator, toml.field_selections)?,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct TomlConfig {
    log_file: Option<PathBuf>,

    #[serde(rename = "initial-env")]
    initial_env_vars: Option<Vec<String>>,

    watched_command: Option<String>,
    interval: Option<f64>,

    #[serde(default)]
    cursor_fg: Color,
    #[serde(default)]
    cursor_bg: Color,
    #[serde(default)]
    cursor_boldness: Boldness,

    header_lines: Option<usize>,
    #[serde(default)]
    header_fg: Color,
    #[serde(default)]
    header_bg: Color,
    #[serde(default)]
    header_boldness: Boldness,

    #[serde(default)]
    non_cursor_non_header_fg: Color,
    #[serde(default)]
    non_cursor_non_header_bg: Color,
    #[serde(default)]
    non_cursor_non_header_boldness: Boldness,

    #[serde(default)]
    selected_bg: Color,

    #[serde(rename = "fields")]
    field_selections: Option<FieldSelections>,
    field_separator: Option<FieldSeparator>,

    keybindings: Option<StringKeybindings>,
}

impl TomlConfig {
    fn parse(config_file: &str) -> Result<Self> {
        let config = toml::from_str(
            &read_to_string(config_file)
                .with_context(|| format!("Failed to read configuration from {config_file}"))?,
        )
        .with_context(|| format!("Failed to parse TOML from {config_file}"))?;
        Ok(config)
    }

    // Merge two configs, where `self` is favored.
    fn merge(self, other: Self) -> Self {
        Self {
            log_file: self.log_file.or(other.log_file),
            initial_env_vars: self.initial_env_vars.or(other.initial_env_vars),
            watched_command: self.watched_command.or(other.watched_command),
            interval: self.interval.or(other.interval),
            non_cursor_non_header_fg: self
                .non_cursor_non_header_fg
                .or(other.non_cursor_non_header_fg),
            non_cursor_non_header_bg: self
                .non_cursor_non_header_bg
                .or(other.non_cursor_non_header_bg),
            non_cursor_non_header_boldness: self
                .non_cursor_non_header_boldness
                .or(other.non_cursor_non_header_boldness),
            cursor_fg: self.cursor_fg.or(other.cursor_fg),
            cursor_bg: self.cursor_bg.or(other.cursor_bg),
            cursor_boldness: self.cursor_boldness.or(other.cursor_boldness),
            header_fg: self.header_fg.or(other.header_fg),
            header_bg: self.header_bg.or(other.header_bg),
            header_boldness: self.header_boldness.or(other.header_boldness),
            selected_bg: self.selected_bg.or(other.selected_bg),
            header_lines: self.header_lines.or(other.header_lines),
            field_separator: self.field_separator.or(other.field_separator),
            field_selections: self.field_selections.or(other.field_selections),
            keybindings: StringKeybindings::merge(self.keybindings, other.keybindings),
        }
    }
}

impl From<ClapConfig> for TomlConfig {
    fn from(clap: ClapConfig) -> Self {
        Self {
            log_file: clap.log_file,
            initial_env_vars: clap.initial_env_vars,
            watched_command: clap.watched_command.map(|s| s.join(" ")),
            interval: clap.interval,
            non_cursor_non_header_fg: clap.non_cursor_non_header_fg,
            non_cursor_non_header_bg: clap.non_cursor_non_header_bg,
            non_cursor_non_header_boldness: clap.non_cursor_non_header_boldness,
            cursor_fg: clap.cursor_fg,
            cursor_bg: clap.cursor_bg,
            cursor_boldness: clap.cursor_boldness,
            header_fg: clap.header_fg,
            header_bg: clap.header_bg,
            header_boldness: clap.header_boldness,
            selected_bg: clap.selected_bg,
            header_lines: clap.header_lines,
            field_separator: clap.field_separator,
            field_selections: clap.field_selections,
            keybindings: clap.keybindings.map(|vec| vec.into()),
        }
    }
}

impl Default for TomlConfig {
    fn default() -> Self {
        let toml = indoc! {r#"
			"interval" = 3.0

			"cursor-fg" = "unspecified"
			"cursor-bg" = "gray"
			"cursor-boldness" = "bold"

			"header-fg" = "blue"
			"header-bg" = "unspecified"
			"header-boldness" = "non-bold"

			"non-cursor-non-header-fg" = "unspecified"
			"non-cursor-non-header-bg" = "unspecified"
			"non-cursor-non-header-boldness" = "unspecified"

			"selected-bg" = "magenta"

			[keybindings]
			"ctrl+c" = [ "exit" ]
			"q" = [ "exit" ]
			"r" = [ "reload" ]
			"?" = [ "help-toggle" ]
			"space" = [ "toggle-selection", "cursor down 1" ]
			"v" = [ "toggle-selection" ]
			"esc" = [ "unselect-all" ]
			"down" = [ "cursor down 1" ]
			"up" = [ "cursor up 1" ]
			"j" = [ "cursor down 1" ]
			"k" = [ "cursor up 1" ]
			"g" = [ "cursor first" ]
			"G" = [ "cursor last" ]
		"#};
        toml::from_str(toml).expect("Default toml config file should be correct")
    }
}

#[derive(Parser)]
#[command(version, about, rename_all = "kebab-case", after_help = Self::all_possible_values())]
pub struct ClapConfig {
    /// Enable logging, and write logs to file.
    #[arg(short, long, value_name = "FILE")]
    log_file: Option<PathBuf>,

    /// Comman-separated `set-env` operations to execute before first watched command execution
    #[arg(long = "initial-env", value_name = "LIST", value_delimiter = ',')]
    initial_env_vars: Option<Vec<String>>,

    /// Command to watch by executing periodically
    #[arg(trailing_var_arg(true))]
    watched_command: Option<Vec<String>>,

    /// TOML config file path
    #[arg(short, long, value_name = "FILE")]
    config_file: Option<String>,

    /// Seconds (f64) to wait between updates, 0 only executes once
    #[arg(short, long, value_name = "SECONDS")]
    interval: Option<f64>,

    /// Foreground color of cursor line
    #[arg(
        long,
        value_name = "COLOR",
        default_value_t,
        hide_default_value = true,
        hide_possible_values = true
    )]
    cursor_fg: Color,

    /// Background color of cursor line
    #[arg(
        long,
        value_name = "COLOR",
        default_value_t,
        hide_default_value = true,
        hide_possible_values = true
    )]
    cursor_bg: Color,

    /// Boldness of cursor line
    #[arg(
        long,
        value_name = "BOLDNESS",
        default_value_t,
        hide_default_value = true,
        hide_possible_values = true
    )]
    cursor_boldness: Boldness,

    /// Foreground color of header lines
    #[arg(
        long,
        value_name = "COLOR",
        default_value_t,
        hide_default_value = true,
        hide_possible_values = true
    )]
    header_fg: Color,

    /// Background color of header lines
    #[arg(
        long,
        value_name = "COLOR",
        default_value_t,
        hide_default_value = true,
        hide_possible_values = true
    )]
    header_bg: Color,

    /// Boldness of header lines
    #[arg(
        long,
        value_name = "BOLDNESS",
        default_value_t,
        hide_default_value = true,
        hide_possible_values = true
    )]
    header_boldness: Boldness,

    /// Foreground color of non-cursor, non-header lines.
    #[arg(
        long,
        value_name = "COLOR",
        default_value_t,
        hide_default_value = true,
        hide_possible_values = true
    )]
    non_cursor_non_header_fg: Color,

    /// Background color of non-cursor, non-header lines.
    #[arg(
        long,
        value_name = "COLOR",
        default_value_t,
        hide_default_value = true,
        hide_possible_values = true
    )]
    non_cursor_non_header_bg: Color,

    /// Boldness of non-cursor, non-header lines.
    #[arg(
        long,
        value_name = "BOLDNESS",
        default_value_t,
        hide_default_value = true,
        hide_possible_values = true
    )]
    non_cursor_non_header_boldness: Boldness,

    /// Background color of selected line indicator
    #[arg(
        long,
        value_name = "COLOR",
        default_value_t,
        hide_default_value = true,
        hide_possible_values = true
    )]
    selected_bg: Color,

    /// The first N lines of the input are treated as a sticky header
    #[arg(long, value_name = "N")]
    header_lines: Option<usize>,

    /// Field separator
    #[arg(short = 's', long, value_name = "STRING")]
    field_separator: Option<FieldSeparator>,

    /// Comma-separated field selections/ranges, e.g. `X`, `X-Y`, `X-` (field indexes start at 1).
    #[arg(short = 'f', long = "fields", value_name = "LIST")]
    field_selections: Option<FieldSelections>,

    // TODO: replace with StringKeybindings once clap supports parsing into HashMap
    // TODO: known clap bug: replace with ClapKeybindings once supported
    /// Keybindings as comma-separated `KEY:OP[+OP]*` pairs, e.g. `q:select+exit,r:reload`.
    #[arg(short = 'b', long = "bind", value_name = "LIST", value_delimiter = ',', value_parser = keybindings::parse_str)]
    keybindings: Option<Vec<(String, Vec<String>)>>,
}

impl ClapConfig {
    /// Get string help menu of all possible values of configuration options.
    fn all_possible_values() -> String {
        let color = PossibleEnumValues::<PrettyColor>::new().get();
        let boldness = PossibleEnumValues::<Boldness>::new().get();
        let key_modifier = PossibleEnumValues::<KeyModifier>::new().hidden().get();
        let key_code = PossibleEnumValues::<KeyCode>::new().custom_names().get();
        let operation = PossibleEnumValues::<OperationParsed>::new()
            .custom_names()
            .get();

        let table_data = [
            ["COLOR", &format!("[{color}]")],
            ["BOLDNESS", &format!("[{boldness}]")],
            ["KEY", "[<KEY-MODIFIER>+<KEY-CODE>, <KEY-CODE>]"],
            ["KEY-MODIFIER", &format!("[{key_modifier}]")],
            ["KEY-CODE", &format!("[{key_code}]")],
            ["OP", &format!("[{operation}]")],
        ];
        let mut table = Builder::from_iter(table_data).build();
        table
            .with(TableStyle::blank())
            // Add left margin for indent.
            .with(Margin::new(2, 0, 0, 0))
            // Remove left padding.
            .with(Padding::new(0, 1, 0, 0));

        // Set table width to terminal width.
        if let Some((TerminalWidth(width), _)) = terminal_size() {
            let width: usize = width.into();
            table
                .with(Width::wrap(width).priority::<PriorityMax>().keep_words())
                .with(Width::increase(width));
        }

        format!("{}\n{table}", "Possible values:".bold().underline())
    }
}
