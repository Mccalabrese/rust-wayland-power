//! Dynamic Media Player Widget (media)
//!
//! A "Smart" widget that interfaces with `playerctl` to control media playback.
//! 
//! Key Features:
//! 1. **Auto-Hiding:** The widget is invisible (`visible = false`) by default and only appears
//!    when an active media player (Spotify, Firefox, mpv, etc.) is detected.
//! 2. **Polling Architecture:** Checks for status updates every 1 second. We use polling instead
//!    of DBus signals here for simplicity and robustness against player crashes.
//! 3. **Universal Control:** Works with any MPRIS-compliant player.

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, Align};
use crate::helpers; // Shared helper for running shell commands

/// Builds the Media Player card.
pub fn build() -> Box {
    // 1. The Container (Hidden by Default)
    // We start with visibility set to FALSE. The polling loop will turn it TRUE
    // only if it successfully gets metadata from a player.
    // This ensures the sidebar doesn't show an empty/broken player.
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(vec!["media-card"])
        .visible(false) // Start hidden
        .halign(Align::Fill)
        .build();

    // 2. Metadata Labels (Title & Artist)
    // We use ellipsize settings to ensure long song titles don't stretch the sidebar
    // or break the layout. They will show as "Song Title..." if too long.
    let title_label = Label::builder()
        .label("Unknown Title")
        .css_classes(vec!["media-title"])
        .wrap(true)
        .max_width_chars(25) // Approx width before wrapping/cutting off
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .halign(Align::Center)
        .build();

    let artist_label = Label::builder()
        .label("Unknown Artist")
        .css_classes(vec!["media-artist"])
        .wrap(true)
        .max_width_chars(25)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .halign(Align::Center)
        .build();

    // 3. Playback Controls (Prev | Play/Pause | Next)
    let controls = Box::builder()
        .orientation(Orientation::Horizontal)
        .halign(Align::Center)
        .spacing(20)
        .margin_top(10)
        .build();

    let btn_prev = Button::builder().label("⏮").css_classes(vec!["media-btn"]).build();
    let btn_play = Button::builder().label("⏸").css_classes(vec!["media-btn", "play-btn"]).build();
    let btn_next = Button::builder().label("⏭").css_classes(vec!["media-btn"]).build();

    // --- Signal Handlers ---
    // These buttons simply fire-and-forget commands to playerctl.
    // We rely on the polling loop to update the UI state (e.g. changing Pause to Play icon).

    btn_prev.connect_clicked(|_| { helpers::run_cmd("playerctl previous"); });
    btn_next.connect_clicked(|_| { helpers::run_cmd("playerctl next"); });
    
    let btn_play_clone = btn_play.clone();
    btn_play.connect_clicked(move |_| { 
        helpers::run_cmd("playerctl play-pause");
        // Note: We don't manually change the icon here. 
        // We let the next poll cycle (max 1s delay) detect the state change.
        // This prevents the UI from getting out of sync if the command fails.
    });

    controls.append(&btn_prev);
    controls.append(&btn_play);
    controls.append(&btn_next);

    container.append(&title_label);
    container.append(&artist_label);
    container.append(&controls);

    // 4. The Polling Loop (State Management)
    // We clone the widget handles so we can modify them inside the closure.
    let container_poll = container.clone();
    let title_poll = title_label.clone();
    let artist_poll = artist_label.clone();
    let play_btn_poll = btn_play_clone.clone();

    // Runs every 1 second
    glib::timeout_add_seconds_local(1, move || {
        // Fetch metadata in a custom format string to minimize parsing logic.
        // Format: "Status;;Title;;Artist" (e.g., "Playing;;Never Gonna Give You Up;;Rick Astley")
        let output = std::process::Command::new("playerctl")
            .arg("metadata")
            .arg("--format")
            .arg("{{status}};;{{title}};;{{artist}}")
            .output();

        match output {
            // Case A: Player Found & Data Retrieved
            Ok(out) if out.status.success() => {
                let raw = String::from_utf8_lossy(&out.stdout);
                let parts: Vec<&str> = raw.trim().split(";;").collect();

                if parts.len() >= 3 {
                    let status = parts[0]; // "Playing", "Paused", or "Stopped"
                    let title = parts[1];
                    let artist = parts[2];

                    // 1. Show the widget
                    container_poll.set_visible(true);

                    // 2. Update Text
                    title_poll.set_label(title);
                    artist_poll.set_label(artist);

                    // 3. Update Play/Pause Icon based on status
                    if status == "Playing" {
                        play_btn_poll.set_label("⏸"); 
                    } else {
                        play_btn_poll.set_label("▶");
                    }
                } else {
                    // Data was malformed or empty -> Hide widget
                    container_poll.set_visible(false);
                }
            },
            // Case B: No Player Found (Command failed)
            _ => {
                // Instantly hide the widget to clear space
                container_poll.set_visible(false);
            }
        }
        // Return Continue to keep the loop running
        glib::ControlFlow::Continue
    });

    container
}
