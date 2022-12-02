use tui::style::{Color, Modifier, Style};
use std::io::{self, Error};

#[derive(Debug)]
pub struct Styles {
	pub style: Style,
	pub highlight_style: Style,
}

pub fn parse_style(
	fg: Option<String>,
	bg: Option<String>,
	fg_plus: Option<String>,
	bg_plus: Option<String>,
	bold: bool,
	bold_sel: bool,
) -> Result<Styles, Error> {
	Ok(Styles {
		style: Style::reset()
			.fg(parse_color(fg)?)
			.bg(parse_color(bg)?)
			.add_modifier(parse_bold(bold)),
		highlight_style: Style::reset()
			.fg(parse_color(fg_plus)?)
			.bg(parse_color(bg_plus)?)
			.add_modifier(parse_bold(bold_sel)),
	})
}

fn parse_bold(bold: bool) -> Modifier {
	if bold {
		Modifier::BOLD
	} else {
		Modifier::empty()
	}
}

fn parse_color(src: Option<String>) -> Result<Color, Error> {
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
			invalid => {
				return Err(io::Error::new(
					io::ErrorKind::Other,
					format!("Invalid color provided: {}", invalid),
				))
			},
		},
		_ => Color::Reset,
	})
}
