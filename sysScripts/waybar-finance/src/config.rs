use std::fs;
use anyhow::{Result, Context};
use crate::app::Config;
use crate::app::App;

pub fn get_config_path() -> Result<std::path::PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Could not find config directory")?;
    Ok(config_dir.join("waybar-finance/config.json"))
}
pub fn load_config(path: &std::path::PathBuf) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(path)
        .context("Failed to read config file")?;
    let config = serde_json::from_str(&content)
        .context("Failed to parse config.json")?;
    Ok(config)
}
pub fn save_config(app: &App) -> Result<()> {
    let config_path = get_config_path()?;
    //make a new config from App state
    let new_config = Config {
        stocks: app.stocks.clone(),
        api_key: app.api_key.clone(),
    };
    //Serialize to pretty JSON
    let json = serde_json::to_string_pretty(&new_config)
        .context("Failed to serialize config")?;
    //Write to disk
    fs::write(config_path, json).context("Failed to write config file")?;
    Ok(())
}
