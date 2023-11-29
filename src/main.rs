mod config;
mod ui;
mod utils;

use anyhow::Result;

use crate::config::Config;
use crate::ui::UI;

/// The name of the application.
const WATCHBIND_NAME: &str = "watchbind";

#[tokio::main]
async fn main() -> Result<()> {
    if let Some(config) = Config::new()? {
        UI::start(config).await?;
    }
    Ok(())
}
