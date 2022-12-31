mod config;
mod ui;
mod command;

use anyhow::Result;

fn main() -> Result<()> {
	ui::start()
}
