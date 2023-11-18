use clap::ValueEnum;
use derive_new::new;
use owo_colors::{AnsiColors as OwoColor, OwoColorize};
use parse_display::{Display, FromStr};
use ratatui::style::{Color as RatatuiColor, Modifier, Style as RatatuiStyle};
use serde::Deserialize;
use std::fmt;
use strum::{EnumIter, IntoEnumIterator};

/// All styles used in the UI.
#[derive(Debug, Clone)]
pub struct Styles {
    /// The style of the line the cursor is on.
    pub cursor: RatatuiStyle,
    /// The style of the header lines.
    pub header: RatatuiStyle,
    /// The style of the lines that the cursor is not on and that are not
    /// header lines.
    pub non_cursor_non_header: RatatuiStyle,
    /// The style of the indicator in selected lines (not the style of the
    /// selected lines themselves).
    pub selected: RatatuiStyle,
}

/// A style encompassing fg, bg and boldness.
#[derive(new)]
pub struct Style {
    /// Foreground color.
    fg: Color,
    /// Background color.
    bg: Color,
    /// Boldness.
    boldness: Boldness,
}

impl Styles {
    /// Create new from individual `Style`s.
    pub fn new(
        non_cursor_style: Style,
        cursor_style: Style,
        header_style: Style,
        selected_style: Style,
    ) -> Self {
        Self {
            non_cursor_non_header: non_cursor_style.into(),
            cursor: cursor_style.into(),
            header: header_style.into(),
            selected: selected_style.into(),
        }
    }
}

impl From<Style> for RatatuiStyle {
    fn from(style: Style) -> Self {
        let mut ratatui_style = RatatuiStyle::default();

        if let Some(fg) = (&style.fg).into() {
            ratatui_style = ratatui_style.fg(fg);
        }
        if let Some(bg) = (&style.bg).into() {
            ratatui_style = ratatui_style.bg(bg);
        }
        match style.boldness {
            Boldness::Bold => ratatui_style = ratatui_style.add_modifier(Modifier::BOLD),
            Boldness::NonBold => ratatui_style = ratatui_style.remove_modifier(Modifier::BOLD),
            Boldness::Unspecified => {}
        }

        ratatui_style
    }
}

/// A wrapper around ratatui's `Color`.
#[derive(Deserialize, FromStr, Display, Clone, Default, ValueEnum, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[display(style = "kebab-case")]
pub enum Color {
    White,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    Reset,
    /// Don't enforce any specific style.
    #[default]
    Unspecified,
}

impl From<&Color> for Option<RatatuiColor> {
    fn from(color: &Color) -> Self {
        match color {
            Color::White => Some(RatatuiColor::White),
            Color::Black => Some(RatatuiColor::Black),
            Color::Red => Some(RatatuiColor::Red),
            Color::Green => Some(RatatuiColor::Green),
            Color::Yellow => Some(RatatuiColor::Yellow),
            Color::Blue => Some(RatatuiColor::Blue),
            Color::Magenta => Some(RatatuiColor::Magenta),
            Color::Cyan => Some(RatatuiColor::Cyan),
            Color::Gray => Some(RatatuiColor::Gray),
            Color::DarkGray => Some(RatatuiColor::DarkGray),
            Color::LightRed => Some(RatatuiColor::LightRed),
            Color::LightGreen => Some(RatatuiColor::LightGreen),
            Color::LightYellow => Some(RatatuiColor::LightYellow),
            Color::LightBlue => Some(RatatuiColor::LightBlue),
            Color::LightMagenta => Some(RatatuiColor::LightMagenta),
            Color::LightCyan => Some(RatatuiColor::LightCyan),
            Color::Reset => Some(RatatuiColor::Reset),
            Color::Unspecified => None,
        }
    }
}

impl Color {
    /// Returns `other` if self is `Unspecified`, otherwise returns self.
    pub fn or(self, other: Self) -> Self {
        match self {
            Color::Unspecified => other,
            color => color,
        }
    }
}

/// A pretty-printable version of `Color` that displays the string
/// representation of a color in its color. Always applies this stylin,
/// even if printing to a terminal.
pub struct PrettyColor(Color);

impl fmt::Display for PrettyColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let colored_color = match Option::<OwoColor>::from(&self.0) {
            Some(owo_color) => self.0.color(owo_color).to_string(),
            None => self.0.to_string(),
        };
        write!(f, "{}", colored_color)?;
        Ok(())
    }
}

// TODO: inefficient, has to unnecessarily collect
impl IntoEnumIterator for PrettyColor {
    type Iterator = std::vec::IntoIter<PrettyColor>;
    fn iter() -> Self::Iterator {
        Color::iter()
            .map(PrettyColor)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl From<&Color> for Option<OwoColor> {
    fn from(color: &Color) -> Self {
        match color {
            Color::White => Some(OwoColor::BrightWhite),
            Color::Black => Some(OwoColor::Black),
            Color::Red => Some(OwoColor::Red),
            Color::Green => Some(OwoColor::Green),
            Color::Yellow => Some(OwoColor::Yellow),
            Color::Blue => Some(OwoColor::Blue),
            Color::Magenta => Some(OwoColor::Magenta),
            Color::Cyan => Some(OwoColor::Cyan),
            Color::Gray => Some(OwoColor::White),
            Color::DarkGray => Some(OwoColor::BrightBlack),
            Color::LightRed => Some(OwoColor::BrightRed),
            Color::LightGreen => Some(OwoColor::BrightGreen),
            Color::LightYellow => Some(OwoColor::BrightYellow),
            Color::LightBlue => Some(OwoColor::BrightBlue),
            Color::LightMagenta => Some(OwoColor::BrightMagenta),
            Color::LightCyan => Some(OwoColor::BrightCyan),
            Color::Reset => None,
            Color::Unspecified => None,
        }
    }
}

/// A wrapper around ratatui's `Modifier::BOLD`.
#[derive(Deserialize, FromStr, Display, Clone, Default, ValueEnum, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[display(style = "kebab-case")]
pub enum Boldness {
    Bold,
    NonBold,
    /// Don't enforce any specific style.
    #[default]
    Unspecified,
}

impl Boldness {
    /// Returns `other` if self is `Unspecified`, otherwise returns self.
    pub fn or(self, other: Self) -> Self {
        match self {
            Boldness::Unspecified => other,
            boldness => boldness,
        }
    }
}
