use tui::style::{
	Color::{self, *},
	Style,
};

#[derive(Default)]
pub struct Styles {
	pub style: Style,
	pub highlight_style: Style,
}

pub fn parse_colors(
	fg: Option<&str>,
	bg: Option<&str>,
	fg_sel: Option<&str>,
	bg_sel: Option<&str>,
) -> Styles {
	let mut style = Style::default();
	style.fg = parse_color(fg);
	style.bg = parse_color(bg);

	let mut highlight_style = Style::default();
	highlight_style.fg = parse_color(fg_sel);
	highlight_style.bg = parse_color(bg_sel);

	Styles {
		style,
		highlight_style,
	}
}

fn parse_color(src: Option<&str>) -> Option<Color> {
	match src {
		Some(color) => match color.to_lowercase().as_str() {
			"white" => Some(White),
			"black" => Some(Black),
			"red" => Some(Red),
			"green" => Some(Green),
			"yellow" => Some(Yellow),
			"blue" => Some(Blue),
			"magenta" => Some(Magenta),
			"cyan" => Some(Cyan),
			"gray" => Some(Gray),
			"dark_gray" => Some(DarkGray),
			"light_red" => Some(LightRed),
			"light_green" => Some(LightGreen),
			"light_yellow" => Some(LightYellow),
			"light_blue" => Some(LightBlue),
			"light_magenta" => Some(LightMagenta),
			"light_cyan" => Some(LightCyan),
			_ => None,
		},
		_ => None,
	}
}
