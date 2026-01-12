
use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::{gdk, Application, ApplicationWindow, Box, Orientation, Align};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use serde_json::Value;
use chrono::{Datelike, Local};

use crate::style;
use crate::helpers;
use crate::media;
use crate::sysinfo;

pub fn build_ui(app: &Application) {
    //Grab screen info
    let display = gdk::Display::default().expect("Could not find a display");
    //Grab first monitor for now, note: Add monitor selection later
    let monitor = display.monitors().item(0)
        .expect("No monitor found")
        .downcast::<gdk::Monitor>()
        .expect("Could not cast to Monitor");

    //Get resolution
    let geometry = monitor.geometry();
    let screen_width = geometry.width();
    let screen_height = geometry.height();
    let calendar_height = (screen_height as f64 * 0.35) as i32;
    //calculate sidebar width
    //For now we'll use 20%
    let dynamic_width = (screen_width as f64 * 0.20) as i32;
    let final_width = std::cmp::max(dynamic_width, 300); //Minimum width of 300px
    
    println!("Detected Screen Width: {}", screen_width);
    println!("Setting Sidebar Width: {}", final_width);

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(final_width)
        .default_height(800)
        .title("My Sidebar")
        .build();
    

    //1. Initialize Layer Shell for the window
    window.init_layer_shell();
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
    //2. Set the layer to Overlay
    window.set_layer(Layer::Overlay);
    window.set_monitor(Some(&monitor));

    // --- HOVER GUARD (Fixes Sway Click-Close Bug) ---
    // We track if the mouse is currently inside the window.
    let is_hovered = Rc::new(RefCell::new(false));

    let hover_controller = gtk4::EventControllerMotion::new();
    let is_hovered_enter = is_hovered.clone();
    let is_hovered_leave = is_hovered.clone();

    hover_controller.connect_enter(move |_, _, _| {
        *is_hovered_enter.borrow_mut() = true;
    });

    hover_controller.connect_leave(move |_| {
        *is_hovered_leave.borrow_mut() = false;
    });
    
    window.add_controller(hover_controller);

    // --- SMART CLOSE LOGIC ---
    let is_hovered_close = is_hovered.clone();

    let launch_time = std::time::Instant::now();
    
    window.connect_is_active_notify(move |win| {
        // Only trigger if the window LOST focus
        if !win.is_active() {
            //Startup Grace Period: Ignore focus losses within first 500ms
            if launch_time.elapsed().as_millis() < 500 {
                return;
            }
            // GUARD: If mouse is still over the window, this is a false positive
            // (common in Sway when clicking buttons). IGNORE IT.
            if *is_hovered_close.borrow() {
                return;
            }

            // Write "Death Note" for the toggle script
            let _ = std::process::Command::new("touch")
                .arg("/tmp/sidebar_just_closed")
                .output();
            
            win.close();
        }
    });
    //3. Anchor it to the Right, Top, and Bottom
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);

    window.set_width_request(final_width);

    style::load_css();
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);

    main_box.set_margin_top(10);
    main_box.set_margin_bottom(10);
    main_box.set_margin_start(10);
    main_box.set_margin_end(10);

    //Top Zone - Quick Toggles
    let top_box = gtk4::Box::new(gtk4::Orientation::Vertical, 15);
    top_box.add_css_class("zone");

    
    // ---- ROW 1 Session Controls ----
    let row_session = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    row_session.set_homogeneous(true);    
    let btn_idle = helpers::make_squared_button("view-conceal-symbolic", "Idle Inhibit");
    let btn_suspend = helpers::make_squared_button("system-suspend-symbolic", "Suspend");
    let btn_lock = helpers::make_squared_button("system-lock-screen-symbolic", "Lock Screen");
    let btn_logout = helpers::make_squared_button("system-log-out-symbolic", "Logout");
    let btn_restart = helpers::make_squared_button("system-reboot-symbolic", "Reboot");
    let btn_power = helpers::make_squared_button("system-shutdown-symbolic", "Power Off");
    
    row_session.append(&btn_idle);
    row_session.append(&btn_suspend);
    row_session.append(&btn_lock);
    row_session.append(&btn_logout);
    row_session.append(&btn_restart);
    row_session.append(&btn_power);

    //---- ROW 2 Toggles ----
    


    let row_toggles = gtk4::Box::new(gtk4::Orientation::Horizontal, 15);
    row_toggles.set_homogeneous(true);

    let btn_radio = helpers::make_icon_button("multimedia-player-symbolic", "Internet Radio");
    let (btn_update, lbl_update_badge) = helpers::make_badged_button("software-update-available-symbolic", "0", "Update System");
    let btn_air = helpers::make_icon_button("airplane-mode-symbolic", "Airplane Mode");
    let btn_dns = helpers::make_icon_button("weather-overcast-symbolic", "Cloudflare DNS");
    let btn_mute = helpers::make_icon_button("audio-volume-muted-symbolic", "Mute Audio");
    let btn_wall = helpers::make_icon_button("image-x-generic-symbolic", "Change Wallpaper");
    let btn_hint = helpers::make_icon_button("emoji-objects-symbolic", "Show Keyhints");
    
    row_toggles.append(&btn_radio);
    row_toggles.append(&btn_wall);
    row_toggles.append(&btn_dns);
    row_toggles.append(&btn_update);
    row_toggles.append(&btn_air);
    row_toggles.append(&btn_mute);
    row_toggles.append(&btn_hint);


    let (box_brightness, scale_brightness) = helpers::make_slider_row("display-brightness-symbolic");
    let (box_volume, scale_volume) = helpers::make_slider_row("audio-volume-high-symbolic");

    top_box.append(&row_session);
    top_box.append(&row_toggles);
    top_box.append(&box_brightness);
    top_box.append(&box_volume);

    //Middle Zone - (Notifications & Finance)
    let middle_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(20)
        .valign(Align::Fill)
        .vexpand(true)
        .build();


    // 1. Add the Media Player
    let media_widget = media::build();
    middle_box.append(&media_widget);

    // 2. System Info (The "Filler")
    // We wrap it in a box that pushes it to the middle vertically
    let sys_widget = sysinfo::build();
    middle_box.append(&sys_widget);
    
    // --- 4. FINANCE ZONE (New Distinct Box) ---
    let finance_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0); // 5px gap
    finance_box.add_css_class("zone"); // Gives it the border/background
    
    // The Main Ticker Label
    let finance_label = gtk4::Label::builder()
        .label("Loading Market Data...")
        .use_markup(true)
        .css_classes(vec!["finance-text".to_string()])
        .justify(gtk4::Justification::Center) // Center the text
        .wrap(false) // Don't auto-wrap, we will force newlines
        .build();

    // The "Click to Launch" Hint
    let finance_hint = gtk4::Label::builder()
        .label("(Click to launch App)")
        .css_classes(vec!["hint-text".to_string()]) // New CSS class
        .build();

    finance_box.append(&finance_label);
    finance_box.append(&finance_hint);

    // Make the WHOLE box clickable
    let click_gesture = gtk4::GestureClick::new();
    finance_box.add_controller(click_gesture.clone());

    //Bottom Zone - Calender
    let bottom_box = gtk4::Box::new(gtk4::Orientation::Vertical, 5);
    bottom_box.add_css_class("zone");
    bottom_box.set_height_request(calendar_height);
    bottom_box.set_vexpand(false);

    // 1. The MAIN Stack (Swaps between Month Grid and Day List)
    let main_stack = gtk4::Stack::new();
    main_stack.set_vexpand(true);
    main_stack.set_valign(gtk4::Align::Fill);
    let stack_switcher = gtk4::StackSwitcher::builder()
        .stack(&main_stack)
        .halign(gtk4::Align::Center)
        .build();

    // --- VIEW 1: MONTH VIEW (Includes Nav Arrows + Grid) ---
    let month_view_box = gtk4::Box::new(gtk4::Orientation::Vertical, 5);
    month_view_box.set_valign(gtk4::Align::Fill); // Keep our expansion fix
    
    // A. The Header
    let nav_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    nav_box.set_halign(gtk4::Align::Center);
    nav_box.set_margin_bottom(10);
    nav_box.set_margin_top(10);

    let btn_prev = gtk4::Button::builder().icon_name("go-previous-symbolic").css_classes(vec!["flat".to_string()]).build();
    let btn_next = gtk4::Button::builder().icon_name("go-next-symbolic").css_classes(vec!["flat".to_string()]).build();
    
    let label_month = gtk4::Label::builder()
        .css_classes(vec!["calendar-title".to_string()])
        .build();

    nav_box.append(&btn_prev);
    nav_box.append(&label_month);
    nav_box.append(&btn_next);

    // B. The Grid Container (Holds JUST the grid so we can swap it)
    let grid_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    grid_container.set_valign(gtk4::Align::Fill);
    grid_container.set_vexpand(true); // Pass expansion down

    month_view_box.append(&nav_box);
    month_view_box.append(&grid_container);

    // --- STATE MANAGEMENT ---
    // We hold the "View Date" in a shared RefCell so buttons can change it
    let current_view_date = Rc::new(RefCell::new(Local::now().date_naive()));

    // Helper closure to redraw the grid based on the current state
    let grid_container_weak = grid_container.clone();
    let label_month_weak = label_month.clone();
    let view_date_state = current_view_date.clone();

    let refresh_grid = move || {
        let date = *view_date_state.borrow();
        
        // 1. Update Title
        label_month_weak.set_label(&date.format("%B %Y").to_string());

        // 2. Clear Old Grid
        while let Some(child) = grid_container_weak.first_child() {
            grid_container_weak.remove(&child);
        }

        // 3. Build & Add New Grid
        let new_grid = helpers::build_calendar_grid(date.year(), date.month());
        grid_container_weak.append(&new_grid);
    };

    // Initial Draw
    refresh_grid();

    // --- NAVIGATION LOGIC ---
    
    // Previous Month (<)
    let view_date_prev = current_view_date.clone();
    let refresh_prev = refresh_grid.clone();
    btn_prev.connect_clicked(move |_| {
        let mut d = *view_date_prev.borrow();
        // Math: Go back one month
        if d.month() == 1 {
            d = d.with_month(12).unwrap().with_year(d.year() - 1).unwrap();
        } else {
            d = d.with_month(d.month() - 1).unwrap();
        }
        *view_date_prev.borrow_mut() = d;
        refresh_prev();
    });

    // Next Month (>)
    let view_date_next = current_view_date.clone();
    let refresh_next = refresh_grid.clone();
    btn_next.connect_clicked(move |_| {
        let mut d = *view_date_next.borrow();
        // Math: Go forward one month
        if d.month() == 12 {
            d = d.with_month(1).unwrap().with_year(d.year() + 1).unwrap();
        } else {
            d = d.with_month(d.month() + 1).unwrap();
        }
        *view_date_next.borrow_mut() = d;
        refresh_next();
    });

    // Add to Stack
    main_stack.add_titled(&month_view_box, Some("month_view"), "Month");

    // --- VIEW 2: DAY VIEW (The Agenda) ---
    let day_view_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    day_view_box.set_margin_top(20);
    day_view_box.set_margin_start(10);

    let now = Local::now();
    
    // Placeholder Content
    let day_title = gtk4::Label::builder()
        .label(format!("Agenda: {}", now.format("%A, %d %b"))) // "Agenda: Friday, 10 Jan"
        .css_classes(vec!["finance-text".to_string()]) // Reuse bold font
        .halign(gtk4::Align::Start)
        .build();
        
    let no_appt_label = gtk4::Label::builder()
        .label("No appointments scheduled.")
        .css_classes(vec!["hint-text".to_string()])
        .halign(gtk4::Align::Start)
        .build();

    day_view_box.append(&day_title);
    day_view_box.append(&no_appt_label);
    
    // Add to Main Stack
    main_stack.add_titled(&day_view_box, Some("day_view"), "Day");


    // --- FINAL ASSEMBLY ---
    bottom_box.append(&stack_switcher);
    bottom_box.append(&main_stack);
    
    // Add to Main Window
    main_box.append(&top_box);
    main_box.append(&middle_box);
    main_box.append(&finance_box);
    main_box.append(&bottom_box);

    window.set_child(Some(&main_box));

    // -- Logic -- session buttons --

    btn_power.connect_clicked(move |_| {
        helpers::run_cmd("systemctl poweroff");
    });

    btn_restart.connect_clicked(move |_| {
        helpers::run_cmd("systemctl reboot");
    });

    // --- SMART LOGOUT LOGIC ---
    btn_logout.connect_clicked(move |_| {
        // 1. Get the current desktop environment name (e.g., "Hyprland", "sway", "Niri")
        let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
        
        // 2. Decide which command to run based on where we are
        let cmd = if desktop.contains("niri") {
            "niri msg action quit"
        } else if desktop.contains("sway") {
            "swaymsg exit"
        } else if desktop.contains("hyprland") {
            "hyprctl dispatch exit"
        } else {
            // 3. Fallback: The "Wlogout Nuclear Option"
            // If we don't recognize the session, just kill the user's processes.
            // This guarantees it works on Gnome, River, or anything else you try later.
            "loginctl terminate-user $USER"
        };
        
        println!("Detected session: '{}'. Running logout command: '{}'", desktop, cmd);
        helpers::run_cmd(cmd);
    });

    btn_suspend.connect_clicked(move |_| {
        helpers::run_cmd("pidof hyprlock >/dev/null || hyprlock & sleep 0.5; systemctl suspend");
    });

    btn_lock.connect_clicked(move |_| {
        helpers::run_cmd(" pidof hyprlock || hyprlock &");
    });

    // --- IDLE INHIBIT (Persistent) ---
    // --- IDLE INHIBIT (Persistent Fix) ---
    // We use a file check command to determine initial state

    // 1. Startup Check: Check if lockfile exists using ls
    // We use std::process because it matches the permissions context of the click handler
    if std::path::Path::new("/tmp/sidebar_idle.lock").exists() {
        println!("Idle Lock found! Activating button.");
        btn_idle.add_css_class("active");
    }

    // 2. Click Handler
    btn_idle.connect_clicked(move |btn| {
        if btn.has_css_class("active") {
            // TURNING OFF
            println!("Disabling Idle Inhibit");
            btn.remove_css_class("active");
            helpers::run_cmd("pkill -CONT hypridle || pkill -CONT swayidle");
            helpers::run_cmd("rm -f /tmp/sidebar_idle.lock");
        } else {
            // TURNING ON
            println!("Enabling Idle Inhibit");
            btn.add_css_class("active");
            helpers::run_cmd("pkill -STOP hypridle || pkill -STOP swayidle");
            helpers::run_cmd("touch /tmp/sidebar_idle.lock");
        }
    });

    // --- Launchers ---
    btn_wall.connect_clicked(move |_| helpers::run_cmd("$HOME/.cargo/bin/wp-select"));
    btn_hint.connect_clicked(move |_| helpers::run_cmd("$HOME/.cargo/bin/kb-launcher"));
    btn_radio.connect_clicked(move |_| helpers::run_cmd("$HOME/.cargo/bin/radio-menu"));

    // --- Cloudflare DNS (Reuse your tools) ---
    let btn_dns_poll = btn_dns.clone();
    btn_dns.connect_clicked(move |_| {
        // 1. Run the toggle logic
        helpers::run_cmd("$HOME/.cargo/bin/cf-toggle");
        
        // 2. Start a "Poller" that checks status every 1 second for 45 seconds.
        // This covers the time it takes you to type the password.
        let btn_target = btn_dns_poll.clone();
        let mut attempts = 0;

        glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
            attempts += 1;

            // Run the status check
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg("$HOME/.cargo/bin/cf-status")
                .output();

            if let Ok(out) = output {
                if let Ok(json) = serde_json::from_slice::<Value>(&out.stdout) {
                    if let Some(class) = json.get("class").and_then(|v| v.as_str()) {
                        // Update the button based on REALITY, not guesses
                        if class == "on" {
                            btn_target.add_css_class("active");
                        } else {
                            btn_target.remove_css_class("active");
                        }
                    }
                }
            }

            // Stop polling after 15 seconds (15 attempts)
            if attempts >= 45 {
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    });

    // ================= SLIDER SYNC (WATCHER) =================
    // This loops every 2 seconds to keep sliders in sync with system changes
    // (e.g. if you use keyboard hotkeys)
    
    let scale_bright_watch = scale_brightness.clone();
    let scale_vol_watch = scale_volume.clone();

    glib::timeout_add_seconds_local(1, move || {
        // 1. Check Brightness
        // Note: We ignore errors to avoid log spam if command fails
        if let Ok(out) = std::process::Command::new("sh").arg("-c").arg("brightnessctl i -m").output() {
            let csv = String::from_utf8_lossy(&out.stdout);
            if let Some(percent_str) = csv.split(',').nth(3) {
                let clean_str = percent_str.replace("%", "").replace("\n", "");
                if let Ok(sys_val) = clean_str.parse::<f64>() {
                    // Only update if significantly different to avoid fighting the user dragging it
                    if (scale_bright_watch.value() - sys_val).abs() > 1.0 {
                        scale_bright_watch.set_value(sys_val);
                    }
                }
            }
        }

        // 2. Check Volume
        if let Ok(out) = std::process::Command::new("sh").arg("-c").arg("wpctl get-volume @DEFAULT_AUDIO_SINK@").output() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Some(vol_str) = stdout.split_whitespace().nth(1) {
                if let Ok(vol_float) = vol_str.parse::<f64>() {
                    let sys_val = vol_float * 100.0;
                    if (scale_vol_watch.value() - sys_val).abs() > 1.0 {
                        scale_vol_watch.set_value(sys_val);
                    }
                }
            }
        }

        glib::ControlFlow::Continue
    });

    // --- Updates (Threaded Fix) ---
    let lbl_update_badge_clone = lbl_update_badge.clone();

    // 1. CLICK: Run the updater (Instant UI feedback)
    btn_update.connect_clicked(move |_| {
        helpers::run_cmd("$HOME/.cargo/bin/updater");
        // Optimistically hide badge
        lbl_update_badge_clone.set_visible(false);
    });

    // 2. CHECK: Poll for updates (THREADED)
    let (update_tx, update_rx) = std::sync::mpsc::channel();
    let lbl_update_target = lbl_update_badge.clone();

    // A. Spawn the heavy worker thread
    std::thread::spawn(move || {
        loop {
            // Run the slow command
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg("$HOME/.cargo/bin/update-check")
                .output();
            
            // Send result to UI
            if let Ok(out) = output {
                let _ = update_tx.send(out.stdout);
            }

            // Sleep for 30 minutes before checking again
            std::thread::sleep(std::time::Duration::from_secs(1800));
        }
    });

    // B. Setup the UI Receiver (Checks mailbox every 1 second)
    glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
        match update_rx.try_recv() {
            Ok(stdout) => {
                // We got a message from the thread!
                if let Ok(json) = serde_json::from_slice::<Value>(&stdout) {
                    if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
                         lbl_update_target.set_label(text);
                         lbl_update_target.set_visible(text != "0");
                    }
                }
            },
            Err(_) => {
                // No message yet, or thread died. Just keep checking.
            }
        }
        glib::ControlFlow::Continue
    });

    // --- Airplane Mode ---
    let btn_air_clone = btn_air.clone();
    btn_air.connect_clicked(move |_| {
        // 1. Actually run the script!
        helpers::run_cmd("$HOME/.cargo/bin/rfkill-manager --toggle");

        // 2. Toggle the visual state immediately (Optimistic UI is fine here)
        if btn_air_clone.has_css_class("active") {
            btn_air_clone.remove_css_class("active");
        } else {
            btn_air_clone.add_css_class("active");
        }
    });
    // --- AIRPLANE STATUS CHECK ---
    let btn_air_status = btn_air.clone();
    glib::MainContext::default().spawn_local(async move {
        // rfkill list returns text. If "Soft blocked: yes", airplane mode is ON.
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg("rfkill list all")
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // If any device is blocked, we consider Airplane Mode "Active"
            if stdout.contains("Soft blocked: yes") {
                btn_air_status.add_css_class("active");
            } else {
                btn_air_status.remove_css_class("active");
            }
        }
    });

    // --- MUTE LOGIC ---
    let btn_mute_clone = btn_mute.clone();
    
    // 1. Click Handler
    btn_mute.connect_clicked(move |_| {
        // Toggle Mute via WirePlumber
        helpers::run_cmd("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle");
        
        // Optimistic UI Update
        if btn_mute_clone.has_css_class("active") {
            btn_mute_clone.remove_css_class("active"); // Unmuted
        } else {
            btn_mute_clone.add_css_class("active"); // Muted (Blue)
        }
    });

    // ================= SLIDER LOGIC =================

    // ================= SLIDER SYNC (DELAYED WATCHER - FIXED) =================
    let scale_bright_watch = scale_brightness.clone();
    let scale_vol_watch = scale_volume.clone();

    glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        
        // FIX: Clone them AGAIN for the inner loop
        let sb_inner = scale_bright_watch.clone();
        let sv_inner = scale_vol_watch.clone();

        // Start the repeating timer (Runs every 1 second)
        glib::timeout_add_seconds_local(1, move || {
            
            // 1. Check Brightness
            if let Ok(out) = std::process::Command::new("brightnessctl").arg("i").arg("-m").output() {
                let csv = String::from_utf8_lossy(&out.stdout);
                if let Some(percent_str) = csv.split(',').nth(3) {
                    let clean_str = percent_str.replace("%", "").replace("\n", "");
                    if let Ok(sys_val) = clean_str.parse::<f64>() {
                         if (sb_inner.value() - sys_val).abs() > 1.0 {
                             sb_inner.set_value(sys_val);
                         }
                    }
                }
            }

            // 2. Check Volume
            if let Ok(out) = std::process::Command::new("sh").arg("-c").arg("wpctl get-volume @DEFAULT_AUDIO_SINK@").output() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Some(vol_str) = stdout.split_whitespace().nth(1) {
                    if let Ok(vol_float) = vol_str.parse::<f64>() {
                        let sys_val = vol_float * 100.0;
                        if (sv_inner.value() - sys_val).abs() > 1.0 {
                            sv_inner.set_value(sys_val);
                        }
                    }
                }
            }

            glib::ControlFlow::Continue
        });

        glib::ControlFlow::Break // Stop the delay timer
    });

    // ================= FINANCE LOGIC (THREAD SAFE FIX) =================

    // 1. Click Handler
    click_gesture.connect_pressed(move |_, _, _, _| {
        helpers::run_cmd("ghostty --title=waybar-finance -e $HOME/.cargo/bin/waybar-finance --tui");
    });

    // 2. Setup Standard Rust Channel
    // We use std::sync::mpsc (Multi-Producer, Single-Consumer)
    let (sender, receiver) = std::sync::mpsc::channel();

    // 3. Spawn Background Thread
    std::thread::spawn(move || {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg("$HOME/.cargo/bin/waybar-finance")
            .output();
        
        // Send data to main thread. If receiver is gone, we don't care.
        let _ = sender.send(output);
    });

    // 4. Poll for Data on Main Thread (Every 100ms)
    let finance_label_update = finance_label.clone();
    
    // We use glib::timeout_add_local to check the receiver repeatedly
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        // Attempt to read from the channel without blocking
        match receiver.try_recv() {
            Ok(Ok(out)) => {
                // SUCCESS: We got data!
                if let Ok(json) = serde_json::from_slice::<Value>(&out.stdout) {
                    if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
                        
                        // --- GRID FORMATTING LOGIC ---
                        let raw_items: Vec<&str> = text.split("</span> ").collect();
                        let mut grid_text = String::new();

                        for (i, item) in raw_items.iter().enumerate() {
                            if item.trim().is_empty() { continue; }
                            
                            grid_text.push_str(item);
                            
                            if !item.ends_with("</span>") {
                                grid_text.push_str("</span>");
                            }

                            // 4 columns per row
                            if (i + 1) % 4 == 0 {
                                grid_text.push('\n');
                            } else {
                                grid_text.push_str("      ");
                            }
                        }
                        
                        finance_label_update.set_markup(&grid_text);

                        if let Some(tt) = json.get("tooltip").and_then(|v| v.as_str()) {
                            finance_label_update.set_tooltip_markup(Some(tt));
                        }
                    }
                }
                // Stop the timer (ControlFlow::Break)
                glib::ControlFlow::Break
            }
            Ok(Err(_)) => {
                // Command failed to execute
                finance_label_update.set_label("Exec Error");
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // Nothing yet, keep waiting (ControlFlow::Continue)
                glib::ControlFlow::Continue
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                // Thread died without sending data
                finance_label_update.set_label("Error (Thread Died)");
                glib::ControlFlow::Break
            }
        }
    });
    // ================= MASTER STATUS LOADER (INSTANT STARTUP) =================
    // We spawn ONE thread to check all system states (DNS, Mute, Air, Sliders)
    // This ensures the window opens in 0.1s, and the toggles pop in 0.5s later.

    let btn_dns_load = btn_dns.clone();
    let btn_air_load = btn_air.clone();
    let btn_mute_load = btn_mute.clone();
    let scale_bright_load = scale_brightness.clone();
    let scale_vol_load = scale_volume.clone();

    let (status_tx, status_rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        // 1. Check DNS
        let dns_out = std::process::Command::new("sh")
            .arg("-c").arg("$HOME/.cargo/bin/cf-status").output().ok();

        // 2. Check Airplane
        let air_out = std::process::Command::new("rfkill").arg("list").arg("all").output().ok();

        // 3. Check Mute
        let mute_out = std::process::Command::new("sh")
            .arg("-c").arg("wpctl get-volume @DEFAULT_AUDIO_SINK@").output().ok();

        // 4. Check Brightness
        let bright_out = std::process::Command::new("brightnessctl").arg("i").arg("-m").output().ok();

        // 5. Check Volume
        // (We already grabbed mute output, but need to parse volume number too)
        
        // Send all results back as a tuple
        let _ = status_tx.send((dns_out, air_out, mute_out, bright_out));
    });

    // Receive and Update UI
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        match status_rx.try_recv() {
            Ok((dns_o, air_o, mute_o, bright_o)) => {
                
                // --- APPLY DNS ---
                if let Some(out) = dns_o {
                    if let Ok(json) = serde_json::from_slice::<Value>(&out.stdout) {
                        if json.get("class").and_then(|v| v.as_str()) == Some("on") {
                            btn_dns_load.add_css_class("active");
                        }
                    }
                }

                // --- APPLY AIRPLANE ---
                if let Some(out) = air_o {
                    let s = String::from_utf8_lossy(&out.stdout);
                    if s.contains("Soft blocked: yes") {
                        btn_air_load.add_css_class("active");
                    }
                }

                // --- APPLY MUTE & VOLUME ---
                if let Some(out) = mute_o {
                    let s = String::from_utf8_lossy(&out.stdout); // "Volume: 0.40 [MUTED]"
                    
                    // Mute State
                    if s.contains("[MUTED]") {
                        btn_mute_load.add_css_class("active");
                    }
                    
                    // Volume Slider
                    if let Some(vol_str) = s.split_whitespace().nth(1) {
                         if let Ok(vol) = vol_str.parse::<f64>() {
                             scale_vol_load.set_value(vol * 100.0);
                         }
                    }
                }

                // --- APPLY BRIGHTNESS ---
                if let Some(out) = bright_o {
                    let s = String::from_utf8_lossy(&out.stdout);
                    if let Some(p) = s.split(',').nth(3) {
                         let clean = p.replace("%", "").replace("\n", "");
                         if let Ok(val) = clean.parse::<f64>() {
                             scale_bright_load.set_value(val);
                         }
                    }
                }

                glib::ControlFlow::Break // Stop polling
            },
            Err(_) => {
                // Keep waiting
                glib::ControlFlow::Continue
            }
        }
    });
    window.present();
}
