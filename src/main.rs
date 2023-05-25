mod command;
mod config;
mod ui;

use crate::config::Config;
use anyhow::Result;

fn main() -> Result<()> {
    ui::start(Config::parse()?)
}
