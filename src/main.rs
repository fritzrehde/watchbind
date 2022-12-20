mod config;
mod ui;
mod exec;

use crate::config::Config;
use crate::ui::start;
use anyhow::Result;

fn main() -> Result<()> {
	start(Config::parse()?)
}
