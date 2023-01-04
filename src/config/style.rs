use anyhow::{bail, Result};
use tui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy)]
pub struct Styles {
	pub line: Style,
	pub cursor: Style,
	pub selected: Style,
}

impl Styles {
	pub fn parse(
		fg: Option<String>,
		bg: Option<String>,
		fg_cursor: Option<String>,
		bg_cursor: Option<String>,
		bg_selected: Option<String>,
		bold: Option<bool>,
		bold_cursor: Option<bool>,
	) -> Result<Self> {
		let new_style = |fg, bg, bold| -> Result<Style> {
			Ok(
				Style::reset()
					.fg(parse_color(fg)?)
					.bg(parse_color(bg)?)
					.add_modifier(parse_bold(bold)),
			)
		};
		Ok(Self {
			line: new_style(fg, bg, bold)?,
			cursor: new_style(fg_cursor, bg_cursor, bold_cursor)?,
			selected: new_style(None, bg_selected, None)?,
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
