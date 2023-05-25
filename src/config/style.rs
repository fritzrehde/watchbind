use anyhow::{bail, Result};
use ratatui::style::{Color, Modifier, Style};

#[derive(Clone)]
pub struct Styles {
    pub line: Style,
    pub cursor: Style,
    pub header: Style,
    pub selected: Style,
}

impl Styles {
    // TODO: clippy says too many arguments (and I agree)
    pub fn parse(
        fg: Option<String>,
        bg: Option<String>,
        bold: Option<bool>,
        cursor_fg: Option<String>,
        cursor_bg: Option<String>,
        cursor_bold: Option<bool>,
        header_fg: Option<String>,
        header_bg: Option<String>,
        header_bold: Option<bool>,
        selected_bg: Option<String>,
    ) -> Result<Self> {
        let new_style = |fg, bg, bold| -> Result<Style> {
            Ok(Style::reset()
                .fg(parse_color(fg)?)
                .bg(parse_color(bg)?)
                .add_modifier(parse_bold(bold)))
        };
        Ok(Self {
            line: new_style(fg, bg, bold)?,
            cursor: new_style(cursor_fg, cursor_bg, cursor_bold)?,
            header: new_style(header_fg, header_bg, header_bold)?,
            selected: new_style(None, selected_bg, None)?,
        })
    }
}

fn parse_bold(bold: Option<bool>) -> Modifier {
    if let Some(true) = bold {
        Modifier::BOLD
    } else {
        Modifier::empty()
    }
}

// TODO: create custom Color type and impl from_str and add parser directly into toml and clap structs
fn parse_color(src: Option<String>) -> Result<Color> {
    Ok(match src {
        Some(color) => match color.to_lowercase().as_str() {
            "white" => Color::White,
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" => Color::Gray,
            "dark_gray" => Color::DarkGray,
            "light_red" => Color::LightRed,
            "light_green" => Color::LightGreen,
            "light_yellow" => Color::LightYellow,
            "light_blue" => Color::LightBlue,
            "light_magenta" => Color::LightMagenta,
            "light_cyan" => Color::LightCyan,
            invalid => bail!("Invalid color provided: \"{}\"", invalid),
        },
        _ => Color::Reset,
    })
}
