mod command;
mod config;
mod ui;

use std::fs::File;

use crate::config::Config;
use anyhow::{Context, Result};
use simplelog::{LevelFilter, WriteLogger};
use ui::UI;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse()?;

    // Setup logging
    if let Some(log_file) = &config.log_file {
        let log_file = File::create(log_file)
            .with_context(|| format!("Failed to create log file: {}", log_file.display()))?;
        let _ = WriteLogger::init(LevelFilter::Info, simplelog::Config::default(), log_file);
    }

    UI::start(config).await
}
