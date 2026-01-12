use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, Align};
use crate::helpers; // We use your existing helper for commands

pub fn build() -> Box {
    // 1. The Container (Hidden by default)
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(vec!["media-card"])
        .visible(false) // Start hidden
        .halign(Align::Fill)
        .build();

    // 2. Metadata Labels
    let title_label = Label::builder()
        .label("Unknown Title")
        .css_classes(vec!["media-title"])
        .wrap(true)
        .max_width_chars(25)
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

    // 3. Controls (Prev | Play/Pause | Next)
    let controls = Box::builder()
        .orientation(Orientation::Horizontal)
        .halign(Align::Center)
        .spacing(20)
        .margin_top(10)
        .build();

    let btn_prev = Button::builder().label("⏮").css_classes(vec!["media-btn"]).build();
    let btn_play = Button::builder().label("⏸").css_classes(vec!["media-btn", "play-btn"]).build();
    let btn_next = Button::builder().label("⏭").css_classes(vec!["media-btn"]).build();

    // Button Actions
    btn_prev.connect_clicked(|_| { helpers::run_cmd("playerctl previous"); });
    btn_next.connect_clicked(|_| { helpers::run_cmd("playerctl next"); });
    
    // Play/Toggle Logic
    let btn_play_clone = btn_play.clone();
    btn_play.connect_clicked(move |_| { 
        helpers::run_cmd("playerctl play-pause");
        // We let the poller update the icon to avoid state desync
    });

    controls.append(&btn_prev);
    controls.append(&btn_play);
    controls.append(&btn_next);

    container.append(&title_label);
    container.append(&artist_label);
    container.append(&controls);

    // 4. The Polling Loop (Checks status every 1s)
    let container_poll = container.clone();
    let title_poll = title_label.clone();
    let artist_poll = artist_label.clone();
    let play_btn_poll = btn_play_clone.clone();

    glib::timeout_add_seconds_local(1, move || {
        // Get metadata in one efficient call: "Status;;Title;;Artist"
        let output = std::process::Command::new("playerctl")
            .arg("metadata")
            .arg("--format")
            .arg("{{status}};;{{title}};;{{artist}}")
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let raw = String::from_utf8_lossy(&out.stdout);
                let parts: Vec<&str> = raw.trim().split(";;").collect();

                if parts.len() >= 3 {
                    let status = parts[0]; // "Playing" or "Paused"
                    let title = parts[1];
                    let artist = parts[2];

                    // UPDATE UI
                    container_poll.set_visible(true);
                    title_poll.set_label(title);
                    artist_poll.set_label(artist);

                    // Update Play/Pause Icon based on status
                    if status == "Playing" {
                        play_btn_poll.set_label("⏸"); 
                    } else {
                        play_btn_poll.set_label("▶");
                    }
                } else {
                    // Weird output? Hide.
                    container_poll.set_visible(false);
                }
            },
            _ => {
                // Command failed (No players found) -> Hide widget
                container_poll.set_visible(false);
            }
        }

        glib::ControlFlow::Continue
    });

    container
}
