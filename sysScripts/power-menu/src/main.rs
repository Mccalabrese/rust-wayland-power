//! Power Menu Manager (power-menu)
//!
//! A dynamic wrapper for `wlogout` that adapts layout and margins based on the 
//! current monitor resolution and scaling factor.
//!
//! Purpose:
//! On Wayland, absolute pixel values in CSS/config don't always scale correctly across
//! mixed-DPI setups (e.g., 4K laptop + 1080p monitor). This tool:
//! 1. Detects the active monitor's logical height.
//! 2. Selects a predefined "Tier" from `config.toml` (e.g., res_2160, res_1080).
//! 3. Calculates precise top/bottom margins to vertically center the menu perfectly.
//! 4. Acts as a toggle: Running it while the menu is open closes it gracefully.

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::process::Command;
use std::fs;

// --- Config Models ---
#[derive(Deserialize, Debug)]
struct MarginConfig {
    top_margin: f64,      // Logical pixels from top
    bottom_margin: f64,   // Logical pixels from bottom
    columns: Option<i32>, // Optional override for column count at this resolution
}
#[derive(Deserialize, Debug)]
struct PowerMenuConfig {
    columns: i32,           // Default column count
    //tiers for different logical heights
    res_2160: MarginConfig, // 4K
    res_1600: MarginConfig, // Ultrawide/Laptop
    res_1440: MarginConfig, // QHD
    res_1080: MarginConfig, // FHD
    res_720: MarginConfig,  // HD
}
#[derive(Deserialize, Debug)]
struct GlobalConfig {
    power_menu: PowerMenuConfig,
}

// --- IPC Response Models ---
#[derive(Deserialize, Debug)]
struct HyprMonitor {
    height: i32,
    scale: f64,
    focused: bool,
}

#[derive(Deserialize, Debug)]
struct SwayOutput {
    focused: bool,
    scale: f64,
    current_mode: SwayMode,
}
#[derive(Deserialize, Debug)]
struct SwayMode {
    height: i32,
}

// --- Environment Detection ---

/// Identifies the active Wayland compositor via IPC sockets or XDG variables.
fn get_compositor() -> String {
    if env::var("NIRI_SOCKET").is_ok() { return "niri".to_string(); }
    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() { return "hyprland".to_string(); }
    if env::var("SWAYSOCK").is_ok() { return "sway".to_string(); }
    // Fallback
    if let Ok(d) = env::var("XDG_CURRENT_DESKTOP") {
        let d = d.to_lowercase();
        if d.contains("niri") { return "niri".to_string(); }
        if d.contains("hypr") { return "hyprland".to_string(); }
        if d.contains("sway") { return "sway".to_string(); }

    }
    "unknown".to_string()
}

fn load_config() -> Result<GlobalConfig> {
    let config_path = dirs::home_dir()
        .context("Cannot find home dir")?
        .join(".config/rust-dotfiles/config.toml");
    let config_str = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file from path {}", config_path.display()))?;
    let config: GlobalConfig = toml::from_str(&config_str)
        .context("Failed to parse config.toml. Check for syntax errors.")?;
    Ok(config)
}

// --- Process Management ---

/// Checks if `wlogout` is already running. If so, kills it and returns true.
/// This implements the "Toggle" behavior (Open -> Close).
fn check_and_kill_wlogout() -> bool {
    let status = Command::new("pkill")
        .arg("-x")
        .arg("wlogout")
        .status();
    match status {
        Ok(s) => s.success(), // True if process found and killed
        Err(_) => false,
    }
}

// --- Resolution Fetchers (Strategy Pattern) ---

/// Queries Hyprland via `hyprctl`.
fn get_hyprland_data() -> Result<(f64, f64)> {
    let output = Command::new("hyprctl")
        .arg("-j")
        .arg("monitors")
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("hyprctl failed");
    }
    
    let monitors: Vec<HyprMonitor> = serde_json::from_slice(&output.stdout)?;
    // I need the focused monitor to ensure the menu opens on the correct screen with correct scaling.
    monitors.iter().find(|m| m.focused)
        .map(|m| (m.height as f64, m.scale))
        .ok_or_else(|| anyhow!("No focused monitor"))
}

/// Queries Sway via `swaymsg`.
fn get_sway_data() -> Result<(f64, f64)> {
    let output = Command::new("swaymsg")
        .arg("-t")
        .arg("get_outputs")
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("swamsg failed");
    }

    let monitors: Vec<SwayOutput> = serde_json::from_slice(&output.stdout)?;

    if let Some(m) = monitors.iter().find(|m| m.focused) {
        // Sway reports raw pixels. We must divide by scale to get logical pixels.
        let logical = (m.current_mode.height as f64) / m.scale;
        Ok((logical, m.scale))
    } else {
        Err(anyhow!("No focused monitor found in swaymsg output"))
    }
}

/// Queries Niri via `niri msg`.
/// Niri output is human-readable text, so we parse with Regex.
fn get_niri_data() -> Result<(f64, f64)> {
    let output = Command::new("niri")
        .arg("msg")
        .arg("outputs")
        .output()?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);

    // Regex extraction
    let mode_re = Regex::new(r"Current mode: (\d+)x(\d+) @")?;
    let scale_re = Regex::new(r"Scale: ([\d\.]+)")?;

    let mode_caps = mode_re.captures(&output_str)
        .context("Could not find 'Current mode:' in niri output")?;
    let scale_caps = scale_re.captures(&output_str)
        .context("Could not find 'Scale:' in niri output")?;

    let height: f64 = mode_caps[2].parse()?;
    let scale: f64 = scale_caps[1].parse()?;

    Ok((height / scale, scale))
}

// --- Layout Calculation ---

/// Determines the correct margins based on Logical Height.
/// This creates a "Responsive Breakpoint" system similar to CSS frameworks.
fn calculate_margins(logical_height: f64, config: &PowerMenuConfig) -> (i32, i32, i32) {
    let (margin_config, default_cols) = if logical_height >= 2160.0 {
        (&config.res_2160, config.columns)
    } else if logical_height >= 1600.0 {
        (&config.res_1600, config.columns)
    } else if logical_height >= 1440.0 {
        (&config.res_1440, config.columns)
    } else if logical_height >= 1080.0 {
        (&config.res_1080, config.columns)
    } else {
        (&config.res_720, config.columns)
    };
    
    let top = margin_config.top_margin as i32;
    let bottom = margin_config.bottom_margin as i32;
    let cols = margin_config.columns.unwrap_or(default_cols);
    
    (top, bottom, cols)
}

fn main() -> Result<()> {
    let global_config = load_config()?;
    let config = global_config.power_menu;

    // Toggle Logic
    if check_and_kill_wlogout() {
        return Ok(()); //Existing instance killed, exit.
    }

    // Environment Data
    let compositor = get_compositor();
    let (logical_height, _scale) = match compositor.as_str() {
        "hyprland" => get_hyprland_data()?,
        "sway" => get_sway_data()?,
        "niri" => get_niri_data()?,
        _ => {
            println!("Unknown compositor, using 1080p defaults.");
            (1080.0, 1.0) // Fallback
        }
    };
    
    // Layout
    let (top, bottom, cols) = calculate_margins(logical_height, &config);

    // Execution
    // We pass margins via command line arguments to override wlogout's CSS defaults.
    println!("Spawning wlogout with T={}, B={}, cols={}", top, bottom, cols);
    Command::new("wlogout")
        .arg("--protocol")
        .arg("layer-shell")
        .arg("-b")
        .arg(cols.to_string())
        .arg("-T")
        .arg(top.to_string())
        .arg("-B")
        .arg(bottom.to_string())
        .spawn()
        .context("Failed to spawn wlogout")?;

    Ok(())
}
