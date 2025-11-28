//! Rfkill Manager (rfkill-manager)
//!
//! A dual-mode utility to control and monitor "Airplane Mode" (rfkill) on Linux.
//! Designed for Waybar integration.
//!
//! Usage:
//!   rfkill-manager --status  => Prints JSON for Waybar (class "on" or "off").
//!   rfkill-manager --toggle  => Switches state, notifies user, and signals Waybar to refresh.

use anyhow::{anyhow, Context, Result};
use notify_rust::Notification;
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn expand_path(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    PathBuf::from(path)
}

// --- Config Modes ---
#[derive(Deserialize, Debug)]
struct RfkillConfig {
    icon: String,
    text_on: String,
    class_on: String,
    tooltip_on: String,
    text_off: String,
    class_off: String,
    tooltip_off: String,
    bar_process_name: String,
    bar_signal_num: i32,
}

#[derive(Deserialize, Debug)]
struct GlobalConfig {
    rfkill_toggle: RfkillConfig,
}

// --- Config Loader (Copied from our other projects) ---
fn load_config() -> Result<GlobalConfig> {
    let config_path = dirs::home_dir()
        .context("Cannot find home dir")?
        .join(".config/rust-dotfiles/config.toml");

    let config_str = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config: {}", config_path.display()))?;

    let config: GlobalConfig = toml::from_str(&config_str)
        .context("Failed to parse config.toml")?;

    Ok(config)
}

// --- System Logic ---

/// Queries the system `rfkill` status.
/// Returns `true` if ANY device is soft-blocked (Airplane Mode is effectively ON).
fn is_blocked() -> Result<bool> {
    let output = Command::new("rfkill")
        .arg("list")
        .arg("all")
        .output()
        .context("Failed to run 'rfkill list'")?;

    if !output.status.success() {
        return Err(anyhow!(
            "rfkill list command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Heuristic: If any device is "Soft blocked: yes", consider Airplane Mode active.
    Ok(stdout.contains("Soft blocked: yes"))
}

// --- Mode: Status (Read-Only) ---

/// Prints the current state in JSON format for Waybar to consume.
fn run_status(config: &RfkillConfig) -> Result<()> {
    // Determine UI state based on system state
    let (text, class, tooltip) = match is_blocked() {
        Ok(true) => (config.text_on.as_str(), config.class_on.as_str(), config.tooltip_on.as_str(),),
        Ok(false) => (config.text_off.as_str(), config.class_off.as_str(), config.tooltip_off.as_str(),),
        Err(e) => {
            eprintln!("rfkill-manager status error: {}", e);
            ("?", "error", "Error checking rfkill")
        }
    };
    println!("{}", json!({
        "text": text,
        "class": class,
        "tooltip": tooltip
    }));
    Ok(())
}

// --- Mode: Toggle (Write) ---

/// Toggles the system state, sends a notification, and refreshes the bar.
fn run_toggle(config: &RfkillConfig) -> Result<()> {
    // Determine Action
    let blocked = is_blocked().context("Failed to check rfkill state before toggle")?;
    let (action, message) = if blocked {
        ("unblock", "Airplane Mode: OFF")
    } else {
        ("block", "Airplane Mode: ON")
    };
    // Execute Change
    let status = Command::new("rfkill")
        .arg(action)
        .arg("all")
        .status()?;
    if !status.success() {
        return Err(anyhow!("rfkill {} command failed", action));
    }
    // Notify User
    let icon_path = expand_path(&config.icon);
    let _ = Notification::new()
        .summary("Airplane Mode")
        .body(message)
        .icon(icon_path.to_str().unwrap_or(""))
        .show();
    
    // 4. Signal Waybar
    // Use a real-time signal (SIGRTMIN + offset) to force Waybar 
    // to re-run the --status command immediately, updating the icon instantly.
    let sig_rtmin = 34; // Standard Linux SIGRTMIN base
    let signal = sig_rtmin + config.bar_signal_num;
    let _ = Command::new("pkill")
        .arg(format!("-{}", signal))
        .arg("-x")
        .arg(&config.bar_process_name)
        .status();
    Ok(())
}
// --- Main Dispatcher ---
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str());
    let config = load_config()?.rfkill_toggle;
    match mode {
        Some("--status") => {
            run_status(&config)?;
        }
        Some("--toggle") | None => {
            if let Err(e) = run_toggle(&config) {
                let _ = Notification::new()
                    .summary("Airplane Mode Error")
                    .body(&e.to_string())
                    .icon("dialog-error")
                    .show();
            }
        }
        _ => {
            println!("Unknown argument. Use --status or --toggle.");
        }
    }
    Ok(())
}
