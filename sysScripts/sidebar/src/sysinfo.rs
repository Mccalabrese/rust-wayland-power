//! System Info Widget (sysinfo)
//!
//! A static "fetch" style widget that displays system information (Kernel, Uptime, Shell, etc.).
//! Designed to be lightweight and battery-friendly by fetching data only once at startup
//! rather than polling continuously.

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, Align};
use std::process::Command;

/// Builds the System Info card widget.
/// Returns a GTK Box containing labeled rows of system data.
pub fn build() -> Box {
    // 1. Container Setup
    // We use a vertical box with specific CSS classes for styling.
    // 'vexpand(true)' ensures this widget pushes adjacent widgets (like the media player)
    // to their respective edges, acting as a visual spacer.
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(vec!["sysinfo-card"])
        .valign(Align::Center)
        .vexpand(true) // Crucial for vertical centering in the sidebar layout
        .build();

    // 2. Data Fetching (Static Snapshot)
    // We execute standard Linux commands to gather system details.
    // NOTE: We do not put this in a loop/thread because these values rarely change
    // during a session (except uptime, but rough accuracy is fine here).
    // This approach consumes zero CPU after the initial load.

    let host = get_stdout("hostname");
    let kernel = get_stdout("uname -r");

    // Shell detection: Safe unwrap with fallbacks
    let shell_path = std::env::var("SHELL").unwrap_or_else(|_| "Unknown".to_string());
    let shell = shell_path.split('/').next_back().unwrap_or("zsh");

    // Session detection: Useful for knowing if we are in Sway, Niri, or Hyprland
    let wm = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Wayland".to_string());
    
    // Package Count: Piped command requires execution via 'sh -c'
    let pkgs = get_stdout("sh -c 'pacman -Q | wc -l'");
    
    // Uptime: 'uptime -p' gives a human-readable string (e.g., "up 2 hours, 10 minutes")
    // We strip the "up " prefix for cleaner UI.
    let uptime = get_stdout("uptime -p").replace("up ", "");

    // 3. Layout Construction
    // Define the data model as a vector of tuples: (Icon + Label, Value)
    let rows = vec![
        ("  Host", host),
        ("  Kernel", kernel),
        ("  Shell", shell.to_string()),
        ("  WM", wm),
        ("📦 Pkgs", pkgs),
        ("  Uptime", uptime),
    ];

    // Iterate and build a row for each data point
    for (icon_label, value) in rows {
        let row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .build();

        // Key Label (Left aligned, expands to push Value to the right)
        let key = Label::builder()
            .label(icon_label)
            .css_classes(vec!["sysinfo-key"])
            .halign(Align::Start)
            .hexpand(true) // Pushes the value label to the far end
            .build();

        // Value Label (Right aligned)
        let val = Label::builder()
            .label(&value)
            .css_classes(vec!["sysinfo-value"])
            .halign(Align::End)
            .build();

        row.append(&key);
        row.append(&val);
        container.append(&row);
    }

    container
}

/// Executes a shell command and returns its trimmed stdout.
/// Handles both simple commands (e.g., "hostname") and complex piped commands (e.g., "sh -c ...").
/// Returns "N/A" on failure instead of panicking to keep the UI stable.
fn get_stdout(cmd: &str) -> String {
    let output = if cmd.contains('\'') {
        // Handle complex piped commands by invoking the shell directly
        Command::new("sh").arg("-c").arg(cmd).output()
    } else {
        // Handle simple commands directly (cleaner process tree)
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        Command::new(parts[0]).args(&parts[1..]).output()
    };

    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => "N/A".to_string(),
    }
}
