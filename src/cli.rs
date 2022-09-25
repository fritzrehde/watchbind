use clap::{command, arg, value_parser, Arg, ArgMatches};
// use crate::config::ConfigRawOptional;

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
				.value_parser(value_parser!(f64)),
		)
		// TODO: create custom value parser
		.arg(
			arg!(keybindings: -b --bind <KEYBINDINGS> "Comma-seperated list of keybindings in the format KEY:CMD[,KEY:CMD]*")
				.required(false)
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
		)
		.arg(
			arg!(--"bg+" <COLOR> "Background color of selected line")
				.required(false)
		)
		.arg(
			arg!(--bold "All lines except selected line are bold")
				.required(false)
		)
		.arg(
			arg!(--"bold+" "Selected line is bold")
				.required(false)
		)
		.arg(
			arg!(-c --config <FILE> "YAML config file path")
				.required(false)
		)
		.get_matches()
}

// pub fn parse_clap() -> (ConfigRawOptional, Option<String>) {
// 	let args = parse_args();

// 	// let config_file = args.value_of("config");
// 	// let config = ConfigRawOptional {
// 	// 	// command: args.values_of("command").or(None).collect::<Vec<&str>>().join(" "),
// 	// 	command: {
// 	// 		match args.values_of("command") {
// 	// 			Some(cmd) => Some(cmd.collect::<Vec<&str>>().join(" ")),
// 	// 			None => None,
// 	// 		}
// 	// 	},
// 	// 	interval: *args.get_one("interval"),
// 	// 	fg: args.value_of("fg"),
// 	// 	bg: args.value_of("bg"),
// 	// 	fg_plus: args.value_of("fg+"),
// 	// 	bg_plus: args.value_of("bg+"),
// 	// 	// TODO: clap returns bool, toml returns Option<bool>, find compromise
// 	// 	bold: args.contains_id("bold"),
// 	// 	bold_plus: args.contains_id("bold+"),
// 	// 	// TODO: fix keybindings
// 	// 	keybindings: Some(Vec::new()),
// 	// };

// 	let config_file = args.get_one::<String>("config");
// 	let config = ConfigRawOptional {
// 		command: {
// 			match args.get_many::<String>("command") {
// 				Some(cmd) => Some(cmd.collect::<Vec<&str>>().join(" ")),
// 				None => None,
// 			}
// 		},
// 		interval: *args.get_one("interval"),
// 		fg: *args.get_one::<String>("fg"),
// 		bg: *args.get_one::<String>("bg"),
// 		fg_plus: *args.get_one::<String>("fg+"),
// 		bg_plus: *args.get_one::<String>("bg+"),
// 		// TODO: clap returns bool, toml returns Option<bool>, find compromise
// 		bold: Some(args.contains_id("bold")),
// 		bold_plus: Some(args.contains_id("bold+")),
// 		// TODO: fix keybindings
// 		keybindings: Some(Vec::new()),
// 	};

// 	(config, config_file)
// }
