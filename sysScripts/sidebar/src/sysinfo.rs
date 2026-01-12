use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, Align};
use std::process::Command;

pub fn build() -> Box {
    // 1. Container
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(vec!["sysinfo-card"])
        .valign(Align::Center)
        .vexpand(true) // Expand to fill the empty space
        .build();

    // 2. Fetch Data
    // We do this once on startup (Static Snapshot) to save battery.
    let host = get_stdout("hostname");
    let kernel = get_stdout("uname -r");
    let shell_path = std::env::var("SHELL").unwrap_or_else(|_| "Unknown".to_string());
    let shell = shell_path.split('/').next_back().unwrap_or("zsh");
    let wm = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Wayland".to_string());
    
    // Package Count (Fast enough to run once)
    let pkgs = get_stdout("sh -c 'pacman -Q | wc -l'");
    
    // Uptime (Pretty format)
    let uptime = get_stdout("uptime -p").replace("up ", "");

    // 3. The "Fetch" Layout (Label: Value)
    let rows = vec![
        ("  Host", host),
        ("  Kernel", kernel),
        ("  Shell", shell.to_string()),
        ("  WM", wm),
        ("📦 Pkgs", pkgs),
        ("  Uptime", uptime),
    ];

    for (icon_label, value) in rows {
        let row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .build();

        let key = Label::builder()
            .label(icon_label)
            .css_classes(vec!["sysinfo-key"])
            .halign(Align::Start)
            .hexpand(true)
            .build();

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

// Helper to run command and get clean string
fn get_stdout(cmd: &str) -> String {
    let output = if cmd.contains('\'') {
        // Handle piped commands like pacman | wc
        Command::new("sh").arg("-c").arg(cmd).output()
    } else {
        // Handle simple commands
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        Command::new(parts[0]).args(&parts[1..]).output()
    };

    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => "N/A".to_string(),
    }
}
