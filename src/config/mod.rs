mod fields;
mod keybindings;
mod style;
mod table;
mod xdg;

use anyhow::{bail, Context, Error, Result};
use clap::Parser;
use indoc::indoc;
use serde::Deserialize;
use simplelog::{LevelFilter, WriteLogger};
use std::{
    borrow::Cow,
    fs::{read_to_string, File},
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use terminal_size::Width;

#[cfg(test)]
use derive_builder::Builder;

use crate::config::keybindings::{KeyCode, KeyModifier};
use crate::config::style::PrettyColor;
use crate::utils::possible_enum_values::PossibleEnumValues;

use self::keybindings::{KeybindingCli, KeybindingsHelpMenuFormat, KeybindingsToml};
use self::style::{Boldness, Color, Style};
use self::{
    fields::{FieldSelections, FieldSeparator},
    keybindings::KeybindingsCli,
};

pub use self::fields::{Fields, TableFormatter};
pub use self::keybindings::{
    KeyEvent, Keybindings, KeybindingsParsed, KeybindingsPrintable, OperationExecutable,
    OperationParsed, Operations, OperationsParsed,
};
pub use self::style::Styles;
pub use self::table::Table;

// TODO: don't have public members

pub struct Config {
    pub log_file: Option<PathBuf>,
    pub watched_command: String,
    pub watch_rate: Duration,
    pub styles: Styles,
    pub keybindings_parsed: KeybindingsParsed,
    pub keybindings_help_menu_format: KeybindingsHelpMenuFormat,
    pub header_lines: usize,
    pub fields: Fields,
    pub initial_env_ops: OperationsParsed,
    pub update_ui_while_blocking: bool,
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

        // If global and/or local config files were provided, parse them
        // into `PartialConfig`s.
        let global_config =
            PartialConfig::parse_from_optional_toml_file(global_config_file, "global")?;
        let local_config =
            PartialConfig::parse_from_optional_toml_file(local_config_file, "local")?;
        let cli_config: PartialConfig = cli_args.try_into()?;
        let default_config = PartialConfig::default();

        let toml_config = PartialConfig::apply_config_overriding_order(
            cli_config,
            local_config,
            global_config,
            default_config,
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

/// Some `PartialConfig` fields **must** be set once everything has been
/// merged. Panic with this error message if that is not the case.
macro_rules! expect {
    ($obj:expr, $field:ident) => {
        $obj.$field.expect(const_format::formatcp!(
            "Expected field '{}' to be set in the default TOML config",
            stringify!($field)
        ))
    };
}

impl TryFrom<PartialConfig> for Config {
    type Error = anyhow::Error;
    fn try_from(config: PartialConfig) -> Result<Self, Self::Error> {
        let non_cursor_non_header_style = Style::new(
            config.non_cursor_non_header_fg,
            config.non_cursor_non_header_bg,
            config.non_cursor_non_header_boldness,
        );
        let cursor_style = Style::new(config.cursor_fg, config.cursor_bg, config.cursor_boldness);
        let header_style = Style::new(config.header_fg, config.header_bg, config.header_boldness);
        let selected_style = Style::new(
            Color::Unspecified,
            config.selected_bg,
            Boldness::Unspecified,
        );
        let styles = Styles::new(
            non_cursor_non_header_style,
            cursor_style,
            header_style,
            selected_style,
        );

        Ok(Self {
            log_file: config.log_file,
            initial_env_ops: config.initial_env_vars.unwrap_or_default().try_into()?,
            watched_command: match config.watched_command {
                Some(command) => command,
                None => bail!("A command must be provided via command line or config file"),
            },
            watch_rate: Duration::from_secs_f64(expect!(config, interval)),
            styles,
            keybindings_parsed: expect!(config, keybindings),
            keybindings_help_menu_format: expect!(config, keybindings_help_menu_format),
            header_lines: expect!(config, header_lines),
            fields: Fields::try_new(config.field_separator, config.field_selections)?,
            update_ui_while_blocking: expect!(config, update_ui_while_blocking),
        })
    }
}

/// A partial configuration that contains all values as optionals, since they
/// may or may not have been set in the configuration source.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Builder), builder(default, pattern = "owned"))]
pub struct PartialConfig {
    log_file: Option<PathBuf>,
    initial_env_vars: Option<Vec<String>>,
    watched_command: Option<String>,
    interval: Option<f64>,
    cursor_fg: Color,
    cursor_bg: Color,
    cursor_boldness: Boldness,
    header_lines: Option<usize>,
    header_fg: Color,
    header_bg: Color,
    header_boldness: Boldness,
    non_cursor_non_header_fg: Color,
    non_cursor_non_header_bg: Color,
    non_cursor_non_header_boldness: Boldness,
    selected_bg: Color,
    field_selections: Option<FieldSelections>,
    field_separator: Option<FieldSeparator>,
    update_ui_while_blocking: Option<bool>,
    keybindings: Option<KeybindingsParsed>,
    keybindings_help_menu_format: Option<KeybindingsHelpMenuFormat>,
}

impl PartialConfig {
    /// Given the `PartialConfig`s from the CLI, possibly from a local config
    /// file, possibly from a global config file, and from the defaults, apply
    /// the config overriding order: `cli > local > global > default`
    /// (where `a > b` means that `a`'s settings override `b`'s on conflicts)
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
            update_ui_while_blocking: self
                .update_ui_while_blocking
                .or(other.update_ui_while_blocking),
            keybindings: KeybindingsParsed::merge(self.keybindings, other.keybindings),
            keybindings_help_menu_format: self
                .keybindings_help_menu_format
                .or(other.keybindings_help_menu_format),
        }
    }

    /// Parse an optional config from an optional TOML config file. The config
    /// file type to be parsed from can be `global` or `local`.
    fn parse_from_optional_toml_file(
        opt_file: Option<&PathBuf>,
        config_file_type: &str,
    ) -> Result<Option<PartialConfig>> {
        match opt_file {
            Some(file) => {
                let config = TomlFileConfig::parse_from_file(file)?
                    .try_into()
                    .with_context(|| {
                        format!(
                            "Failed to parse {} TOML config file located at {}",
                            config_file_type,
                            file.display()
                        )
                    })?;
                Ok(Some(config))
            }
            None => Ok(None),
        }
    }
}

/// A configuration originating from a TOML config file.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct TomlFileConfig {
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

    update_ui_while_blocking: Option<bool>,

    keybindings: Option<KeybindingsToml>,

    keybindings_help_menu_format: Option<KeybindingsHelpMenuFormat>,
}

impl TomlFileConfig {
    /// Parse a `TomlFileConfig` from the a TOML `file`.
    fn parse_from_file<P: AsRef<Path>>(file: P) -> Result<Self> {
        let config_str = read_to_string(&file).with_context(|| {
            format!(
                "Failed to read configuration from {}",
                file.as_ref().display()
            )
        })?;
        config_str.parse().with_context(|| {
            format!(
                "Invalid TOML syntax when reading configuration from {}",
                file.as_ref().display()
            )
        })
    }
}

impl FromStr for TomlFileConfig {
    type Err = anyhow::Error;
    fn from_str(config_str: &str) -> Result<Self, Self::Err> {
        toml::from_str(config_str).context("Failed to parse TOML string into TomlConfig")
    }
}

impl TryFrom<TomlFileConfig> for PartialConfig {
    type Error = Error;
    fn try_from(toml: TomlFileConfig) -> Result<Self> {
        Ok(Self {
            log_file: toml.log_file,
            initial_env_vars: toml.initial_env_vars,
            watched_command: toml.watched_command,
            interval: toml.interval,
            non_cursor_non_header_fg: toml.non_cursor_non_header_fg,
            non_cursor_non_header_bg: toml.non_cursor_non_header_bg,
            non_cursor_non_header_boldness: toml.non_cursor_non_header_boldness,
            cursor_fg: toml.cursor_fg,
            cursor_bg: toml.cursor_bg,
            cursor_boldness: toml.cursor_boldness,
            header_fg: toml.header_fg,
            header_bg: toml.header_bg,
            header_boldness: toml.header_boldness,
            selected_bg: toml.selected_bg,
            header_lines: toml.header_lines,
            field_separator: toml.field_separator,
            field_selections: toml.field_selections,
            update_ui_while_blocking: toml.update_ui_while_blocking,
            keybindings: toml
                .keybindings
                .map(KeybindingsParsed::try_from)
                .transpose()?,
            keybindings_help_menu_format: toml.keybindings_help_menu_format,
        })
    }
}

impl TryFrom<CliArgs> for PartialConfig {
    type Error = Error;
    fn try_from(cli: CliArgs) -> Result<Self> {
        Ok(Self {
            log_file: cli.log_file,
            initial_env_vars: cli.initial_env_vars,
            watched_command: cli.watched_command.map(|s| s.join(" ")),
            interval: cli.interval,
            non_cursor_non_header_fg: cli.non_cursor_non_header_fg,
            non_cursor_non_header_bg: cli.non_cursor_non_header_bg,
            non_cursor_non_header_boldness: cli.non_cursor_non_header_boldness,
            cursor_fg: cli.cursor_fg,
            cursor_bg: cli.cursor_bg,
            cursor_boldness: cli.cursor_boldness,
            header_fg: cli.header_fg,
            header_bg: cli.header_bg,
            header_boldness: cli.header_boldness,
            selected_bg: cli.selected_bg,
            header_lines: cli.header_lines,
            field_separator: cli.field_separator,
            field_selections: cli.field_selections,
            update_ui_while_blocking: cli.update_ui_while_blocking,
            keybindings: cli
                .keybindings
                .map(KeybindingsCli::from)
                .map(KeybindingsParsed::try_from)
                .transpose()?,
            keybindings_help_menu_format: cli.keybindings_help_menu_format,
        })
    }
}

// TODO: add test that checks that the default config sets all values.
impl Default for PartialConfig {
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

            "update-ui-while-blocking" = false

            "keybindings-help-menu-format" = [ "key", "description", "operations" ]

            [keybindings]
            "ctrl+c" = { description = "Exit watchbind", operations = "exit" }
            "q" = { description = "Exit watchbind", operations = "exit" }
            "r" = { description = "Reload the watched command manually, resets interval timer", operations = "reload" }

            # Moving around
            "down" = { description = "Move cursor down 1 line", operations = "cursor down 1" }
            "up" = { description = "Move cursor up 1 line", operations = "cursor up 1" }
            "j" = { description = "Move cursor down 1 line", operations = "cursor down 1" }
            "k" = { description = "Move cursor up 1 line", operations = "cursor up 1" }
            "g" = { description = "Move cursor to the first line", operations = "cursor first" }
            "G" = { description = "Move cursor to the last line"  , operations = "cursor last" }

            # Selecting lines
            "space" = { description = "Toggle selection of line that cursor is currently on, and move cursor down 1 line", operations = [ "toggle-selection", "cursor down 1" ] }
            "v" = { description = "Select line that cursor is currently on", operations = "select" }
            "esc" = { description = "Unselect all currently selected lines", operations = "unselect-all" }

            # Help menu
            "?" = { description = "Toggle the visibility of the help menu", operations = "help-toggle" }
		"#};

        default_toml
            .parse::<TomlFileConfig>()
            .expect("Default embedded toml config file should have correct TOML syntax")
            .try_into()
            .expect("Default embedded toml config file should contain valid settings")
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

    /// Whether to update the UI with new output from the watched command
    /// while in a blocking state.
    #[arg(long, value_name = "BOOL")]
    update_ui_while_blocking: Option<bool>,

    /// Keybindings as comma-separated `KEY:OP[+OP]*` pairs, e.g. `q:select+exit,r:reload`.
    #[arg(short = 'b', long = "bind", value_name = "LIST", value_delimiter = ',')]
    keybindings: Option<Vec<KeybindingCli>>,

    /// Format of keybindings help menu as comma-separated list, e.g. `key,operations,description`.
    #[arg(long, value_name = "FORMAT")]
    keybindings_help_menu_format: Option<KeybindingsHelpMenuFormat>,
}

/// Convert [[&str, String]] to [[Cow::Borrowed(&str), Cow::Owned(&str)]].
macro_rules! cowify {
    ($([$str_slice:expr, $string:expr]),* $(,)?) => {
        [$(
            [Cow::Borrowed($str_slice), Cow::Owned($string)],
        )*]
    };
}

fn terminal_width() -> Option<u16> {
    terminal_size::terminal_size().map(|(Width(width), _)| width)
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

        let possible_values_table_data = cowify![
            ["COLOR", format!("[{color}]")],
            ["BOLDNESS", format!("[{boldness}]")],
            ["KEY", format!("[<KEY-MODIFIER>+<KEY-CODE>, <KEY-CODE>]")],
            ["KEY-MODIFIER", format!("[{key_modifier}]")],
            ["KEY-CODE", format!("[{key_code}]")],
            ["OP", format!("[{operation}]")],
        ];
        let possible_values_table = Table::new(possible_values_table_data)
            .width(terminal_width())
            .left_margin(2)
            .displayable();

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

        let global_config_file: Cow<str> = global_config_file_path()
            .map_or(Cow::Borrowed("Unknown"), |file| {
                Cow::Owned(file.display().to_string())
            });

        let global_config_file_table_data = [[global_config_file.as_ref()]];
        let global_config_file_table = Table::new(global_config_file_table_data)
            .width(terminal_width())
            .left_margin(2)
            .displayable();

        // Mimic clap's bold underlined style for headers.
        format!(
            "{}\n{}",
            "Global config file:".bold().underline(),
            global_config_file_table,
        )
    }
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
        let cli = PartialConfigBuilder::default()
            .interval(Some(3.0))
            .cursor_bg(Color::Blue)
            .selected_bg(Color::Yellow)
            .header_lines(None)
            .build()
            .unwrap();

        let local = PartialConfigBuilder::default()
            .interval(Some(2.0))
            .cursor_fg(Color::Gray)
            .header_bg(Color::Magenta)
            .header_lines(None)
            .build()
            .unwrap();

        let global = PartialConfigBuilder::default()
            .cursor_bg(Color::Red)
            .cursor_fg(Color::Green)
            .header_lines(Some(4))
            .build()
            .unwrap();

        let default = PartialConfigBuilder::default()
            .selected_bg(Color::Black)
            .header_bg(Color::Red)
            .header_lines(Some(5))
            .build()
            .unwrap();

        let merged = PartialConfig::apply_config_overriding_order(
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
