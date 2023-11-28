mod boldness;
mod color;

use derive_new::new;
use ratatui::style::{Modifier, Style as RatatuiStyle};

pub use self::boldness::Boldness;
pub use self::color::{Color, PrettyColor};

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

        if let Some(fg) = style.fg.into() {
            ratatui_style = ratatui_style.fg(fg);
        }
        if let Some(bg) = style.bg.into() {
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
