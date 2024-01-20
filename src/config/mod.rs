mod fields;
mod keybindings;
mod style;
mod xdg;

use anyhow::{bail, Context, Result};
use clap::Parser;
use indoc::indoc;
use serde::Deserialize;
use simplelog::{LevelFilter, WriteLogger};
use std::{
    fs::{read_to_string, File},
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use tabled::settings::{peaker::PriorityMax, Margin, Padding, Style as TableStyle, Width};
use tabled::{builder::Builder, Table};
use terminal_size::{terminal_size, Width as TerminalWidth};

#[cfg(test)]
use derive_builder::Builder;

use crate::config::keybindings::{KeyCode, KeyModifier};
use crate::config::style::PrettyColor;
use crate::utils::possible_enum_values::PossibleEnumValues;

use self::fields::{FieldSelections, FieldSeparator};
use self::keybindings::{KeybindingsParsed, StringKeybindings};
use self::style::{Boldness, Color, Style};

pub use self::fields::{Fields, TableFormatter};
pub use self::keybindings::{
    KeyEvent, Keybindings, Operation, OperationParsed, Operations, OperationsParsed,
};
pub use self::style::Styles;

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

const GLOBAL_CONFIG_FILE: &str = "config.toml";

impl Config {
    /// Build a new `Config` from CLI options, local and global config files,
    /// and default values.
    pub fn new() -> Result<Self> {
        let cli_args = CliArgs::parse();

        // Setup logging, if requested.
        if let Some(log_file) = &cli_args.log_file {
            Self::setup_logging(log_file)?;
        }

        let global_config_file_path = global_config_file_path()?;
        let global_config_file: Option<&PathBuf> = (global_config_file_path.is_file()
            && global_config_file_path.exists())
        .then_some(&global_config_file_path);
        let local_config_file: Option<&PathBuf> = cli_args.local_config_file.as_ref();

        // If global and/or local config files were provided, parse them into `TomlConfig`s.
        let global_toml = global_config_file
            .map(TomlConfig::parse)
            .transpose()
            .with_context(|| {
                format!(
                    "Failed to parse global toml config file located at {}",
                    global_config_file_path.display()
                )
            })?;
        let local_toml = local_config_file
            .map(TomlConfig::parse)
            .transpose()
            .with_context(|| {
                format!(
                    "Failed to parse local toml config file located at {}",
                    global_config_file_path.display()
                )
            })?;
        let cli_toml: TomlConfig = cli_args.into();
        let default_toml = TomlConfig::default();

        dbg!(&global_toml);

        let toml_config = TomlConfig::apply_config_overriding_order(
            cli_toml,
            local_toml,
            global_toml,
            default_toml,
        );

        toml_config.try_into()
    }

    /// Configure the logger to save logs to a `log_file`.
    fn setup_logging<P: AsRef<Path>>(log_file: P) -> Result<()> {
        let log_file = File::create(&log_file).with_context(|| {
            format!("Failed to create log file: {}", log_file.as_ref().display())
        })?;
        WriteLogger::init(LevelFilter::Info, simplelog::Config::default(), log_file)?;
        Ok(())
    }
}

/// Get the global config file path, regardless of whether the file actually
/// exists.
fn global_config_file_path() -> Result<PathBuf> {
    let mut global_config_dir = xdg::config_dir()?;
    global_config_dir.push(GLOBAL_CONFIG_FILE);
    Ok(global_config_dir)
}

impl TryFrom<TomlConfig> for Config {
    type Error = anyhow::Error;
    fn try_from(toml: TomlConfig) -> Result<Self, Self::Error> {
        let non_cursor_non_header_style = Style::new(
            toml.non_cursor_non_header_fg,
            toml.non_cursor_non_header_bg,
            toml.non_cursor_non_header_boldness,
        );
        let cursor_style = Style::new(toml.cursor_fg, toml.cursor_bg, toml.cursor_boldness);
        let header_style = Style::new(toml.header_fg, toml.header_bg, toml.header_boldness);
        let selected_style =
            Style::new(Color::Unspecified, toml.selected_bg, Boldness::Unspecified);
        let styles = Styles::new(
            non_cursor_non_header_style,
            cursor_style,
            header_style,
            selected_style,
        );

        // Some fields **must** contain a value in `TomlConfig::default()`.
        // Panic with this error message if that is not the case.
        let error_msg = "Should have a value in the default toml config";

        Ok(Self {
            log_file: toml.log_file,
            initial_env_ops: toml.initial_env_vars.unwrap_or_default().try_into()?,
            watched_command: match toml.watched_command {
                Some(command) => command,
                None => bail!("A command must be provided via command line or config file"),
            },
            watch_rate: Duration::from_secs_f64(toml.interval.expect(error_msg)),
            styles,
            keybindings_parsed: toml.keybindings.expect(error_msg).try_into()?,
            header_lines: toml.header_lines.expect(error_msg),
            fields: Fields::try_new(toml.field_separator, toml.field_selections)?,
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Builder), builder(default, pattern = "owned"))]
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
    /// Parse a `TomlConfig` from the string content of a `config_file`.
    fn parse<P: AsRef<Path>>(config_file: P) -> Result<Self> {
        let config_str = read_to_string(&config_file).with_context(|| {
            format!(
                "Failed to read configuration from {}",
                config_file.as_ref().display()
            )
        })?;
        config_str.parse()
    }

    /// Given the `TomlConfig`s from the CLI, possibly from a local config file,
    /// possibly from a global config file, and from the defaults, apply the
    /// config overriding order: `cli > local > global > default`
    /// (where `a > b` means that a's settings override b's on conflicts)
    fn apply_config_overriding_order(
        cli: Self,
        local: Option<Self>,
        global: Option<Self>,
        default: Self,
    ) -> Self {
        match (local, global) {
            (Some(local), Some(global)) => cli.merge(local.merge(global)),
            (Some(local), None) => cli.merge(local),
            (None, Some(global)) => cli.merge(global),
            (None, None) => cli,
        }
        .merge(default)
    }

    /// Merge two configs, where `self` is favored over `other`.
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

impl FromStr for TomlConfig {
    type Err = anyhow::Error;
    fn from_str(config_str: &str) -> Result<Self, Self::Err> {
        toml::from_str(config_str).context("Failed to parse TOML string into TomlConfig")
    }
}

impl From<CliArgs> for TomlConfig {
    fn from(clap: CliArgs) -> Self {
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
        let default_toml = indoc! {r#"
			"interval" = 3.0

			"cursor-fg" = "unspecified"
			"cursor-bg" = "blue"
			"cursor-boldness" = "bold"

			"header-fg" = "blue"
			"header-bg" = "unspecified"
			"header-boldness" = "non-bold"
            "header-lines" = 0

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
        toml::from_str(default_toml).expect("Default embedded toml config file should be correct")
    }
}

#[derive(Parser)]
#[command(version, about, rename_all = "kebab-case", after_help = Self::extra_help_menu())]
pub struct CliArgs {
    /// Enable logging, and write logs to file.
    #[arg(short, long, value_name = "FILE")]
    log_file: Option<PathBuf>,

    /// Comman-separated `set-env` operations to execute before first watched command execution
    #[arg(long = "initial-env", value_name = "LIST", value_delimiter = ',')]
    initial_env_vars: Option<Vec<String>>,

    /// Command to watch by executing periodically
    #[arg(trailing_var_arg(true))]
    watched_command: Option<Vec<String>>,

    /// File path to local TOML config file
    #[arg(short = 'c', long, value_name = "FILE")]
    local_config_file: Option<PathBuf>,

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

/// Convert [[&str, String]] to [[String, String]] by calling str::to_owned().
macro_rules! to_owned_first {
    ($([$str_slice:expr, $string:expr]),* $(,)?) => {
        [$(
            [str::to_owned($str_slice), $string],
        )*]
    };
}

impl CliArgs {
    /// Get extra help menu as string.
    fn extra_help_menu() -> String {
        format!(
            "{}\n\n{}",
            Self::all_possible_values_help(),
            Self::global_config_file_help()
        )
    }

    /// Get string help menu of all possible values of configuration options.
    fn all_possible_values_help() -> String {
        use owo_colors::OwoColorize;

        let color = PossibleEnumValues::<PrettyColor>::new().get();
        let boldness = PossibleEnumValues::<Boldness>::new().get();
        let key_modifier = PossibleEnumValues::<KeyModifier>::new().hidden().get();
        let key_code = PossibleEnumValues::<KeyCode>::new().custom_names().get();
        let operation = PossibleEnumValues::<OperationParsed>::new()
            .custom_names()
            .get();

        let possible_values_table_data = to_owned_first![
            ["COLOR", format!("[{color}]")],
            ["BOLDNESS", format!("[{boldness}]")],
            ["KEY", format!("[<KEY-MODIFIER>+<KEY-CODE>, <KEY-CODE>]")],
            ["KEY-MODIFIER", format!("[{key_modifier}]")],
            ["KEY-CODE", format!("[{key_code}]")],
            ["OP", format!("[{operation}]")],
        ];
        let possible_values_table = create_table_from(possible_values_table_data);

        // Mimic clap's bold underlined style for headers.
        format!(
            "{}\n{}",
            "Possible values:".bold().underline(),
            possible_values_table,
        )
    }

    /// Get string help menu of the global config file.
    fn global_config_file_help() -> String {
        use owo_colors::OwoColorize;

        let global_config_file = global_config_file_path()
            .map_or("Unknown".to_string(), |file| file.display().to_string());

        let global_config_file_table_data = [[global_config_file]];
        let global_config_file_table = create_table_from(global_config_file_table_data);

        // Mimic clap's bold underlined style for headers.
        format!(
            "{}\n{}",
            "Global config file:".bold().underline(),
            global_config_file_table,
        )
    }
}

/// Create a formatted `tabled::Table` with two columns.
fn create_table_from<R, C>(table_data: R) -> Table
where
    R: IntoIterator<Item = C>,
    C: IntoIterator<Item = String>,
{
    let mut table = Builder::from_iter(table_data.into_iter().map(|row| row.into_iter())).build();
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

    table
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Assert `a > b` by checking that `a`'s `attribute` persisted during the
    /// merging of `a` and `b`.
    macro_rules! assert_a_overrides_b_on_attribute {
        ($src_a:expr, $src_b:expr, $attribute:ident, $merged:expr) => {
            assert_ne!(
                $src_a.$attribute,
                $src_b.$attribute,
                "Compared attributes must differ before merging, because it is impossible to observe overriding behaviour during merges if the overriding and overridden attributes are the same."
            );
            assert_eq!($src_a.$attribute, $merged.$attribute);
        };
    }

    #[test]
    fn test_toml_config_overriding_order() {
        let cli = TomlConfigBuilder::default()
            .interval(Some(3.0))
            .cursor_bg(Color::Blue)
            .selected_bg(Color::Yellow)
            .header_lines(None)
            .build()
            .unwrap();

        let local = TomlConfigBuilder::default()
            .interval(Some(2.0))
            .cursor_fg(Color::Gray)
            .header_bg(Color::Magenta)
            .header_lines(None)
            .build()
            .unwrap();

        let global = TomlConfigBuilder::default()
            .cursor_bg(Color::Red)
            .cursor_fg(Color::Green)
            .header_lines(Some(4))
            .build()
            .unwrap();

        let default = TomlConfigBuilder::default()
            .selected_bg(Color::Black)
            .header_bg(Color::Red)
            .header_lines(Some(5))
            .build()
            .unwrap();

        let merged = TomlConfig::apply_config_overriding_order(
            cli.clone(),
            Some(local.clone()),
            Some(global.clone()),
            default.clone(),
        );

        assert_a_overrides_b_on_attribute!(cli, local, interval, merged);
        assert_a_overrides_b_on_attribute!(cli, global, cursor_bg, merged);
        assert_a_overrides_b_on_attribute!(cli, default, selected_bg, merged);
        assert_a_overrides_b_on_attribute!(local, global, cursor_fg, merged);
        assert_a_overrides_b_on_attribute!(local, default, header_bg, merged);
        assert_a_overrides_b_on_attribute!(global, default, header_lines, merged);
    }
}
