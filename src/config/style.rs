use anyhow::{bail, Result};
use tui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy)]
pub struct Styles {
	pub line: Style,
	pub cursor: Style,
	pub selected: Style,
}

pub fn parse_style(
	fg: Option<String>,
	bg: Option<String>,
	fg_cursor: Option<String>,
	bg_cursor: Option<String>,
	bg_selected: Option<String>,
	bold: bool,
	bold_cursor: bool,
) -> Result<Styles> {
	Ok(Styles {
		line: Style::reset()
			.fg(parse_color(fg)?)
			.bg(parse_color(bg)?)
			.add_modifier(parse_bold(bold)),
		cursor: Style::reset()
			.fg(parse_color(fg_cursor)?)
			.bg(parse_color(bg_cursor)?)
			.add_modifier(parse_bold(bold_cursor)),
		selected: Style::reset().bg(parse_color(bg_selected)?),
	})
}

fn parse_bold(bold: bool) -> Modifier {
	if bold {
		Modifier::BOLD
	} else {
		Modifier::empty()
	}
}

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
