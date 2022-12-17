mod config;
mod exec;
mod keybindings;
mod state;
mod style;
mod terminal_manager;
mod tui;

use crate::config::Config;
use anyhow::Result;

fn main() -> Result<()> {
	tui::start(Config::parse()?)
}
