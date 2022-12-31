mod command;
mod config;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
	ui::start()
}
