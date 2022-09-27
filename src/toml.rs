use config::{File, Config};
use crate::config::ConfigRawFile;

pub fn parse_toml(config_file: &str) -> ConfigRawFile {
	Config::builder()
		.add_source(File::with_name(config_file))
		.build().unwrap()
		.try_deserialize()
		.expect("Error occured while parsing toml config file")
}