use anyhow::{Context, Result};
use std::{env, path::PathBuf};

use crate::WATCHBIND_NAME;

/// Environment variable the user can set to override default config directory.
const WATCHBIND_CONFIG_DIR: &str = "WATCHBIND_CONFIG_DIR";

/// Get watchbind's default configuration directory.
pub fn config_dir() -> Result<PathBuf> {
    Ok(match user_config_dir() {
        Some(user_config_dir) => user_config_dir,
        None => {
            // TODO: find cleaner syntax sugar (.join() isn't used because less efficient)
            let mut config_dir = default_os_config_dir()?;
            config_dir.push(WATCHBIND_NAME);
            config_dir
        }
    })
}

/// Get the user-configured OS configuration directory, if available.
fn user_config_dir() -> Option<PathBuf> {
    env::var_os(WATCHBIND_CONFIG_DIR).map(PathBuf::from)
}

/// Get the default OS configuration directory.
fn default_os_config_dir() -> Result<PathBuf> {
    let default_os_config_dir = if cfg!(target_os = "macos") {
        // On MacOS, use `$HOME/.config` instead of dirs::config_dir().
        dirs::home_dir().map(|h| h.join(".config"))
    } else {
        dirs::config_dir()
    }
    .context("failed to find default OS config directory")?;

    Ok(default_os_config_dir)
}
