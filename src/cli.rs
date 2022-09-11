use clap::{command, arg, value_parser, Arg, ArgMatches};

const DEFAULT_INTERVAL: &str = "5"; // watch interval in s
const DEFAULT_BINDINGS: &str = "q:exit,esc:unselect,down:next,up:previous,j:next,k:previous,g:first,G:last";

// TODO: set required(false) as default
pub fn parse_args() -> ArgMatches {
	command!()
		.trailing_var_arg(true)
		.arg(
			Arg::new("command")
				.help("Command to execute periodically")
				.multiple_values(true)
				.required(true)
				.value_parser(value_parser!(String)),
		)
		.arg(
			arg!(-i --interval <SECS> "Seconds to wait between updates, 0 only executes once")
				.required(false)
				.default_value(DEFAULT_INTERVAL)
				.value_parser(value_parser!(f64)),
		)
		// TODO: create custom value parser
		.arg(
			arg!(-b --bind <KEYBINDINGS> "Comma-seperated list of keybindings in the format KEY:CMD[,KEY:CMD]*")
				.id("keybindings")
				.required(false)
				.default_value(DEFAULT_BINDINGS)
				.value_parser(value_parser!(String))
		)
		.arg(
			arg!(--fg <COLOR> "Foreground color")
				.required(false)
		)
		.arg(
			arg!(--bg <COLOR> "Background color")
				.required(false)
		)
		.arg(
			arg!(--"fg+" <COLOR> "Foreground color of selected line")
				.required(false)
				.default_value("black")
		)
		.arg(
			arg!(--"bg+" <COLOR> "Background color of selected line")
				.required(false)
				.default_value("blue")
		)
		.get_matches()
}
