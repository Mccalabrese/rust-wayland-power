use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};
use crate::app::Config;

/// Resolves the XDG-compliant configuration path.
/// Usually ~/.config/waybar-finance/config.json on Linux.
pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Could not find config directory")?;
    Ok(config_dir.join("waybar-finance/config.json"))
}
/// Loads the configuration from disk.
/// Returns a default configuration if the file does not exist.
pub fn load_config(path: &PathBuf) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(path)
        .context("Failed to read config file")?;
    let config = serde_json::from_str(&content)
        .context("Failed to parse config.json")?;
    Ok(config)
}
/// Persists the current application state to config.json.
/// This handles creating the directory structure if it doesn't exist (first run).
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;
    let json = serde_json::to_string_pretty(config)
        .context("Failed to serialize config")?;
    
    // Ensure the parent directory exists before writing.
    // This fixes crashes on fresh installs where ~/.config/waybar-finance/ is missing.
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }
    //Write to disk
    fs::write(config_path, json).context("Failed to write config file")?;
    Ok(())
}
