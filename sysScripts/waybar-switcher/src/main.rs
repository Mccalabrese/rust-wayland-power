//! Waybar Configuration Switcher
//!
//! A system utility that automatically detects the running Wayland compositor 
//! (Niri, Hyprland, or Sway) and hot-swaps the corresponding Waybar configuration file.
//!
//! This solves the problem of using a single status bar across multiple window managers
//! where layout requirements (modules, workspaces) differ significantly.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::thread;
use std::time::Duration;

/// Expands the tilde (`~`) in file paths to the user's home directory.
/// Rust's standard library `Path` does not handle shell expansions automatically.
fn expand_path(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    PathBuf::from(path)
}

// --- Config Models ---

#[derive(Deserialize, Debug)]
struct WaybarSwitcherConfig {
    target_file: String,    // The active config file read by Waybar
    niri_config: String,    // Source file for Niri
    hyprland_config: String,// Source file for Hyprland
    sway_config: String,    // Source file for Sway
}

#[derive(Deserialize, Debug)]
struct GlobalConfig {
    waybar_switcher: WaybarSwitcherConfig,
}

// --- Config Management ---

/// Loads the mapping of compositors to config files from the central dotfiles config.
fn load_config() -> Result<GlobalConfig> {
    let config_path = dirs::home_dir().context("Cannot find home dir")?.join(".config/rust-dotfiles/config.toml");

    let config_str = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file from path: {}", config_path.display()))?;

    let config: GlobalConfig = toml::from_str(&config_str)
        .context("Failed to parse config.toml. Check for syntax errors.")?;
    
    Ok(config)
}

/// Identifies the active Wayland compositor by checking unique environment variables.
///
/// I prioritize socket variables (e.g., `SWAYSOCK`) over `XDG_CURRENT_DESKTOP`
/// because the latter is sometimes set incorrectly by display managers or previous sessions.
fn get_compositor() -> Option<String> {
    // Check for specific IPC sockets first (most reliable method)
    if env::var("NIRI_SOCKET").is_ok() {
        return Some("niri".to_string());
    }
    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return Some("hyprland".to_string());
    }
    if env::var("SWAYSOCK").is_ok() {
        return Some("sway".to_string());
    }
    // Fallback: Check standard XDG variables
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        let desktop = desktop.to_lowercase();
        if desktop.contains("niri") { return Some("niri".to_string()); }
        if desktop.contains("hyprland") { return Some("hyprland".to_string()); }
        if desktop.contains("sway") { return Some("sway".to_string()); }
    }
    None
}
fn main() -> Result<()> {
    // 1. Load User Preferences
    let global_config = load_config()?;
    let config = global_config.waybar_switcher;
    //2.Detect Environment
    let compositor = get_compositor().unwrap_or_else(|| "unknown".to_string());
    println!("Detected compositor: {}", compositor);
    // 3. Select Config Source
    // I map the detected environment to the specific source file defined in config.toml.
    let source_path_str = match compositor.as_str() {
        "niri" => &config.niri_config,
        "hyprland" => &config.hyprland_config,
        "sway" => &config.sway_config,
        _ => {
            println!("Unknown compositor, defaulting to Hyprland config.");
            &config.hyprland_config
        }
    };
    // Expand paths to handle `~/` notation from the TOML file
    let source_path = expand_path(source_path_str);
    let target_path = expand_path(&config.target_file);

    println!("Copying config:\n  From: {:?}\n  To:   {:?}", source_path, target_path);

    // 4. Overwrite Active Configuration
    // We overwrite the target file rather than symlinking to avoid issues 
    // where file watchers might track the link target instead of the link itself.
    fs::copy(&source_path, &target_path)
        .with_context(|| format!("Failed to copy {:?} to {:?}", source_path, target_path))?;

    // 5. Restart Waybar Process
    println!("Restarting Waybar...");
    // Kill existing instances to prevent duplicates or zombie processes.
    // We ignore the result because it fails if Waybar isn't running, which is fine.
    let _ = Command::new("pkill").arg("-x").arg("waybar").status();
    // Brief sleep to ensure the socket is released by the OS before restarting.
    thread::sleep(Duration::from_millis(500));
    // Spawn new instance detached from this process
    Command::new("waybar")
        .arg("-c")
        .arg(&target_path)
        .spawn()
        .context("Failed to spawn new waybar process")?;

    println!("Waybar restarted successfully.");
    Ok(())
}
