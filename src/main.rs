mod command;
mod config;
mod ui;

use anyhow::Result;
use crate::config::Config;

fn main() -> Result<()> {
	ui::start(Config::parse()?)
}
