//! System Update Wrapper (sys-update)
//!
//! A robust automation tool for Arch Linux system maintenance.
//! 1. Reads configuration from `~/.config/rust-dotfiles/config.toml`.
//! 2. Verifies that necessary binaries (`ghostty`, `yay`, etc.) exist before execution.
//! 3. Wraps the package manager (`yay`/`pacman`) in a GUI terminal window so the user can see progress and enter `sudo` passwords.
//! 4. Chains system updates with firmware updates (`fwupdmgr`).
//! 5. Provides desktop notifications on success/failure using `notify-rust`.

use std::fs;
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Context, Result};
use notify_rust::{Notification, Urgency};
use serde::Deserialize;

const LOGO: &str = r#"
"++++++++++
     ++++++++++++++
    ++++++++++++++++
   ++++++++++++++++++
  ++++++++++++++++++++
 +++++++++====+++++++++
 ++++++=:......:=++++++
 +++++=:..........:=+++++
 ++++=..............=++++
 +++=.=##=......=##-.=+++
++++:-%%-.-....-%%:.-:++++
+++=.*%%. *....#%%..*.=+++
+++-.#%%#*%....%%%###.-+++
+++-.#%%%%#....#%%%%#.-+++
+++-.+%%%%*....*%%%%+.-+++
 ++=.:#%%#:....:#%%#:.=++
 +++..:=+:......:+=:..+++
++++-................-++++
+++++:..............:+++++
"#;

/// Expands shell-style paths like `~/` to absolute system paths.
fn expand_path(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    PathBuf::from(path)
}
// 🐧🐧🐧 Config Models 🐧🐧🐧

#[derive(Deserialize, Debug)]
struct Global {
    terminal: String, // The user's preferred terminal emulator
}

#[derive(Deserialize, Debug)]
struct UpdaterConfig {
    update_command: Vec<String>, //The actual update command (e.g. "yay", "-Syu")
    icon_success: String,        //Path to success icon
    icon_error: String,          // Path to error icon
    window_title: String,        // Title for the window manager to target rules
}

#[derive(Deserialize, Debug)]
struct GlobalConfig {
    global: Global,
    updater: UpdaterConfig,
}

/// Loads and parses the TOML configuration file.
/// Centralizes all settings so recompilation isn't needed for minor changes.
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

// 🐧🐧🐧 Helper Functions 🐧🐧🐧
/// Checks if a binary is executable in the current $PATH.
/// Used for "Fail Fast" validation before launching the GUI.
fn check_dependency(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null()) // Suppress output
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
/// Sends a desktop notification via D-Bus.
fn send_notification(summary: &str, body: &str, icon: &Path, urgency: Urgency) -> Result<()> {
    Notification::new()
        .summary(summary)
        .body(body)
        .icon(icon.to_str().unwrap_or(""))
        .urgency(urgency)
        .show()
        .context("Failed to send desktop notification")?;
    Ok(())
}

// --- Main Execution Flow ---

fn main() -> Result<()> {
    // Load Configuration
    let config = load_config()?;
    let global_conf = config.global;
    let updater_conf = config.updater;

    // Resolve relative paths
    let icon_error = expand_path(&updater_conf.icon_error);
    let icon_success = expand_path(&updater_conf.icon_success);
    
    // Dependency Verification
    let terminal_cmd = &global_conf.terminal;
    let update_bin = updater_conf.update_command.first().context("update_command empty")?;

    if !check_dependency(terminal_cmd) { return Err(anyhow!("Terminal not found: {}", terminal_cmd)); }
    if !check_dependency(update_bin) { return Err(anyhow!("Update helper not found: {}", update_bin)); }
    
    let update_cmd_str = updater_conf.update_command.join(" ");
    
    // --- CONSTRUCT THE BASH SCRIPT ---
    // We use a raw string literal (r#...#) so we can write Bash naturally.
    let bash_script = format!(r#"
        cat << "EOF"
{}
EOF
        echo -e "\n🚀 Starting System Update..."
        
        # --- 1. SYSTEM UPDATE ---
        {}
        sys_exit=$?

        # --- 2. FIRMWARE UPDATE ---
        if [ $sys_exit -eq 0 ]; then
            echo -e "\n\n🔌 Checking for Firmware Updates..."
            if command -v fwupdmgr &> /dev/null; then
                sudo fwupdmgr refresh > /dev/null
                if fwupdmgr get-updates 2>&1 | grep -q "No updates"; then
                    echo "✔ Firmware is up to date."
                else
                    echo -e "\n⚠️  Firmware updates available! Updating..."
                    sudo fwupdmgr update
                fi
            else
                echo "fwupdmgr not found, skipping."
            fi
        else
            echo -e "\n⚠ System update failed, skipping firmware/scripts."
        fi

        # --- 3. REFRESH CONFIGS & PACKAGES ---
        if [ $sys_exit -eq 0 ]; then
            echo -e "\n\n🔄 Refreshing Session Configs..."
            
            # A. Run Installer in "Refresh Mode"
            # This handles: 
            #   1. Deleting unwanted Gnome/UWSM sessions
            #   2. Renaming Niri/Hyprland sessions
            #   3. Regenerating Sway-Hybrid wrapper (if Nvidia detected)
            INSTALLER_BIN="$HOME/.cargo/bin/install-wizard"
            
            if [ -f "$INSTALLER_BIN" ]; then
                 sudo "$INSTALLER_BIN" --refresh-configs
            else
                 echo "⚠️ Installer binary not found. Skipping config refresh."
                 echo "Run 'cargo build --release' in sysScripts/install-wizard to fix."
            fi

            # B. Install New Packages from Repo Root
            PKG_FILE="$HOME/rust-wayland-power/pkglist.txt"
            if [ -f "$PKG_FILE" ]; then
                echo -e "\n📦 Checking for new packages in pkglist.txt..."
                # Bash trick: grep removes comments, pacman installs differences
                grep -v "^#" "$PKG_FILE" | sudo pacman -S --needed --noconfirm -
            else
                echo "⚠️ pkglist.txt not found at $PKG_FILE"
            fi
        fi

        # --- 4. RUST TOOLS SELF-UPDATE ---
        if [ $sys_exit -eq 0 ]; then
            echo -e "\n\n🦀 Checking for Rust Script Updates..."
            if [ -d "$HOME/rust-wayland-power/.git" ]; then
                cd "$HOME/rust-wayland-power"
                
                # Check for local changes to avoid overwriting work
                if ! git diff --quiet sysScripts || ! git diff --cached --quiet sysScripts; then
                    echo -e "\n⚠️  Local changes detected in sysScripts! Skipping update."
                else
                    echo "Fetching remote..."
                    git fetch origin main
                    
                    if ! git diff --quiet HEAD..origin/main -- sysScripts; then
                        echo -e "\n✨ Updates detected! Syncing..."
                        git checkout origin/main -- sysScripts
                        
                        echo "🔨 Recompiling Toolchain..."
                        cd sysScripts
                        for dir in */; do
                            if [ -f "$dir/Cargo.toml" ]; then
                                echo "   >> Compiling $dir..."
                                (cd "$dir" && cargo install --path . --force --quiet)
                            fi
                        done
                        echo -e "✅ Custom tools updated."
                    else
                        echo "✔ Rust tools are up to date."
                    fi
                fi
            fi
        fi

        echo -e "\n\n🏁 Process finished. Closing in 5s..."
        sleep 5

        if [ $sys_exit -ne 0 ]; then exit 1; else exit 0; fi
        "#,
        LOGO,
        update_cmd_str
    );

    // Interactive Execution
    let status = Command::new(terminal_cmd)
        .arg(format!("--title={}", updater_conf.window_title))
        .arg("-e")
        .arg("bash")
        .arg("-c")
        .arg(&bash_script)
        .status()
        .context(format!("Failed to launch terminal: {}", terminal_cmd))?;
    
    // Notifications
    if status.success() {
        send_notification("System Update Complete", "All updates applied successfully.", &icon_success, Urgency::Low)?;
    } else {
        send_notification("System Update Failed", "The update process encountered an error.", &icon_error, Urgency::Critical)?;
    }
    Ok(())
}

