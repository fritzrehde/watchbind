use clap::ValueEnum;
use owo_colors::AnsiColors as OwoColor;
use parse_display::{Display, FromStr};
use ratatui::style::Color as RatatuiColor;
use serde::Deserialize;
use std::fmt;
use strum::{EnumIter, IntoEnumIterator};

/// A wrapper around ratatui's `Color`.
#[derive(Debug, Deserialize, FromStr, Display, Clone, Default, ValueEnum, EnumIter)]
#[cfg_attr(test, derive(PartialEq))]
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

impl Color {
    /// Returns `other` if self is `Unspecified`, otherwise returns `self`.
    pub fn or(self, other: Self) -> Self {
        match self {
            Color::Unspecified => other,
            color => color,
        }
    }
}

impl From<Color> for Option<RatatuiColor> {
    fn from(color: Color) -> Self {
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

/// A pretty-printable version of `Color` that displays the string
/// representation of a color in its color. Always applies this styling,
/// even if printed to a terminal.
pub struct PrettyColor(Color);

impl fmt::Display for PrettyColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use owo_colors::OwoColorize;

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
