use std::env;
use std::fs;
use std::process::Command;
use anyhow::{Context, Result};
use serde::Deserialize; 
use toml; 

// --- 1. Config Structs (Same as status.rs) ---
#[derive(Deserialize, Debug)]
struct Config {
    //Ignored fields but serde is making me take them
    text_on: String,
    class_on: String,
    text_off: String,
    class_off: String,
    //the actual ones I need
    resolv_content_on: String,
    resolv_content_off: String,
    bar_process_name: String,
    bar_signal_num: i32,
}

#[derive(Deserialize, Debug)]
struct GlobalConfig {
    cloudflare_toggle: Config,
}

// --- 2. Config Loader (Same as status.rs) ---
fn load_config() -> Result<GlobalConfig> {
    let config_path = dirs::home_dir()
        .context("Cannot find home dir")?
        .join(".config/rust-dotfiles/config.toml");
    let config_str = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file from path: {}", config_path.display()))?;
    let config: GlobalConfig = toml::from_str(&config_str)
        .context("Failed to parse config.toml. Check for syntax errors.")?;
    Ok(config)
}

// --- 3. run_as_user ---
fn run_as_user() -> Result<()> {
    // Load config for Waybar signal
    let config = load_config()
        .context("Failed to load config for user")?
        .cloudflare_toggle;

    // Check status 
    let is_running = Command::new("systemctl")
        .arg("is-active")
        .arg("cloudflared-dns")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    
    let mode = if is_running { "--stop" } else { "--start" };
    let content_on = &config.resolv_content_on;
    let content_off = &config.resolv_content_off;
    // Run pkexec 
    let self_exe = env::current_exe()
        .context("Failed to get path to own executable")?;
    let status = Command::new("pkexec")
        .arg(self_exe)
        .arg(mode)
        .arg(content_on)
        .arg(content_off)
        .status()
        .context("Failed to run pkexec")?;

    // Send signal HARDCODING SIGRTMIN to 34!!!!
    if status.success() {
        let sig_base = 34;
        let signal = sig_base + config.bar_signal_num;
        let _ = Command::new("pkill")
            .arg(format!("-{}", signal))
            .arg("-x")
            .arg(&config.bar_process_name)
            .status();
    }
    Ok(())
}

// --- 4. run_as_root ---
fn run_as_root(mode: &str, content_on: &str, content_off: &str) -> Result<()> {
    if mode == "--start" {
        // Start service
        Command::new("systemctl")
            .arg("enable")
            .arg("--now")
            .arg("cloudflared-dns")
            .status()?
            .success()
            .then_some(())
            .context("Failed to start systemctl service")?;

        // Write resolv.conf from config
        fs::write("/etc/resolv.conf", content_on)
            .context("Failed to write /etc/resolv.conf")?;

    } else if mode == "--stop" {
        // Stop service 
        Command::new("systemctl")
            .arg("disable")
            .arg("--now")
            .arg("cloudflared-dns")
            .status()?
            .success()
            .then_some(())
            .context("Failed to stop systemctl service")?;
        
        // Write resolv.conf from config
        fs::write("/etc/resolv.conf", content_off)
            .context("Failed to write /etc/resolv.conf")?;
    }
    Ok(())
}

// --- 5. Main ---
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let mode = &args[1];
        if mode != "--start" && mode != "--stop" {
            if args.len() < 4 {
                eprintln!("Internal Error: Missing arguments for root mode.");
                return Ok(());
            }
            let content_on = &args[2];
            let content_off = &args[3];
            run_as_root(mode, content_on, content_off)
        } else {
            let content_on = args.get(2).context("Missing content_on")?;
            let content_off = args.get(3).context("Missing content_off")?;
            run_as_root(mode, content_on, content_off)
        }
    } else {
        run_as_user()
    }
}
