use clap::{command, value_parser, Arg, ArgMatches};

pub fn parse_args() -> ArgMatches {
	command!()
		.arg(Arg::new("command")
				 .help("Input command to execute periodically")
				 .required(true))
		.arg(Arg::new("interval")
				 .long("interval")
				 .short('i')
				 .help("Seconds to wait between updates")
				 .takes_value(true)
				 .value_name("SECS")
				 .value_parser(value_parser!(u64)))
		.get_matches()
}
