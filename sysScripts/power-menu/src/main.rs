use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::Deserialize;
use serde_json;
use std::env;
use std::process::Command;
use toml;
use std::fs;

// --- Config Structs (from config.toml) ---
#[derive(Deserialize, Debug)]
struct MarginConfig {
    top_margin: f64,
    bottom_margin: f64,
    columns: Option<i32>,
}
#[derive(Deserialize, Debug)]
struct PowerMenuConfig {
    columns: i32,
    res_2160: MarginConfig,
    res_1600: MarginConfig,
    res_1440: MarginConfig,
    res_1080: MarginConfig,
    res_720: MarginConfig,
}
#[derive(Deserialize, Debug)]
struct GlobalConfig {
    power_menu: PowerMenuConfig,
}

// --- Hyprland JSON Struct ---
#[derive(Deserialize, Debug)]
struct HyprMonitor {
    height: i32,
    scale: f64,
    focused: bool,
}

// --- Sway JSON Structs ---
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

// --- Helper: Get Compositor ---
fn get_compositor() -> String {
    if env::var("NIRI_SOCKET").is_ok() { return "niri".to_string(); }
    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() { return "hyprland".to_string(); }
    if env::var("SWAYSOCK").is_ok() { return "sway".to_string(); }
    if let Ok(d) = env::var("XDG_CURRENT_DESKTOP") {
        let d = d.to_lowercase();
        if d.contains("niri") { return "niri".to_string(); }
        if d.contains("hypr") { return "hyprland".to_string(); }
        if d.contains("sway") { return "sway".to_string(); }

    }
    "unknown".to_string()
}

// --- Helper: Load Config ---
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

// --- Kill Existing wlogout ---
fn check_and_kill_wlogout() -> bool {
    let status = Command::new("pkill")
        .arg("-x")
        .arg("wlogout")
        .status();
    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

// --- Data Fetcher: Hyprland (JSON) ---
fn get_hyprland_data() -> Result<(f64, f64)> {
    let output = Command::new("hyprctl")
        .arg("-j")
        .arg("monitors")
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("hyprctl failed");
    }
    
    let monitors: Vec<HyprMonitor> = serde_json::from_slice(&output.stdout)?;
    
    monitors.iter().find(|m| m.focused)
        .map(|m| (m.height as f64, m.scale))
        .ok_or_else(|| anyhow!("No focused monitor"))
}

// --- Data Fetcher: Sway (JSON) ---
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
        let logical = (m.current_mode.height as f64) / m.scale;
        Ok((logical, m.scale))
    } else {
        Err(anyhow!("No focused monitor found in swaymsg output"))
    }
}

// --- Data Fetcher: Niri (Regex) ---
fn get_niri_data() -> Result<(f64, f64)> {
    let output = Command::new("niri")
        .arg("msg")
        .arg("outputs")
        .output()?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);

    // Niri doesn't specify focused, so we parse the first monitor
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

// --- The Math ---
// This function takes the logical height and picks the right margin from config
fn calculate_margins(logical_height: f64, config: &PowerMenuConfig) -> (i32, i32, i32) {
    
    // Find the right config "tier" based on the logical height
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
    
    // I just used the margins directly as logical pixels.
    // wlogout will handle scaling them based on the monitor's scale factor.
    let top = margin_config.top_margin as i32;
    let bottom = margin_config.bottom_margin as i32;
    
    // Use the tier-specific column count, or fall back to the global default
    let cols = margin_config.columns.unwrap_or(default_cols);
    
    (top, bottom, cols)
}

// --- Main Function ---
fn main() -> Result<()> {
    // 1. Load config
    let global_config = load_config()?;
    let config = global_config.power_menu;

    // 2. Check/kill existing wlogout
    if check_and_kill_wlogout() {
        return Ok(()); // Toggled off, exit gracefully
    }

    // 3. Get compositor
    let compositor = get_compositor();

    // 4. Get normalized data: (LogicalHeight, Scale)
    let (logical_height, _scale) = match compositor.as_str() {
        "hyprland" => get_hyprland_data()?,
        "sway" => get_sway_data()?,
        "niri" => get_niri_data()?,
        _ => {
            println!("Unknown compositor, using 1080p defaults.");
            (1080.0, 1.0) // Fallback
        }
    };
    
    // 5. Do the math (which is just selecting the tier)
    let (top, bottom, cols) = calculate_margins(logical_height, &config);

    // 6. Spawn wlogout with calculated args
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
