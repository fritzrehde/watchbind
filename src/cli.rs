use clap::{command, value_parser, Arg, ArgMatches};

pub fn parse_args() -> ArgMatches {
	command!()
		.trailing_var_arg(true)
		// .allow_hyphen_values(true)
		.arg(Arg::new("command") // TODO: use all of last values instead of one
				 .help("Input command to execute periodically")
				 .value_parser(value_parser!(String))
				 .multiple_values(true)
				 .required(true))
		.arg(Arg::new("interval")
				 .long("interval")
				 .short('i')
				 .help("Seconds to wait between updates, 0 only executes once")
				 .takes_value(true)
				 .value_name("SECS")
				 .value_parser(value_parser!(u64)))
		.arg(Arg::new("keybindings")
				 .long("bind")
				 .short('b')
				 .help("Keybindings in the format <key>:<cmd>,<key>:<cmd>")
				 .takes_value(true)
				 .value_name("key1:cmd1,key2:cmd2"))
		.get_matches()
}
