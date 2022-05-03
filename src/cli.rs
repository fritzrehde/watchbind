use clap::{command, Arg, ArgMatches};

pub fn parse_args() -> ArgMatches {
	command!()
		.arg(Arg::new("command")
				 .help("Input command to execute periodically")
				 .required(true))
		.arg(Arg::new("interval")
				 .long("interval")
				 .short('i')
				 .value_name("SECS")
				 .help("Seconds to wait between updates")
				 .takes_value(true))
		.get_matches()
}
