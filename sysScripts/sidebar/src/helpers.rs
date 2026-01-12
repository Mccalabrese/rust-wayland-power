use gtk4::prelude::*;
use chrono::{Datelike, Local, NaiveDate};

pub fn make_squared_button(icon_name: &str, tooltip: &str) -> gtk4::Button {
    let icon = gtk4::Image::builder()
        .icon_name(icon_name)
        .pixel_size(20)
        .build();
    gtk4::Button::builder()
        .child(&icon)
        .css_classes(vec!["squared-btn".to_string()])
        .height_request(20)
        .tooltip_text(tooltip)
        .build()
}

pub fn make_icon_button(icon_name: &str, tooltip: &str) -> gtk4::Button {
    // Create the image part first, so we can control the size
    let icon = gtk4::Image::builder()
        .icon_name(icon_name)
        .pixel_size(24) // Now this works!
        .build();

    gtk4::Button::builder()
        .child(&icon) // Use .child() instead of .icon_name()
        .css_classes(vec!["circular-btn".to_string()]) // Fix string types here too
        .height_request(30)
        .tooltip_text(tooltip)
        .build()
        
}
// 2. NEW: The Badged Button Helper (For Updates)
pub fn make_badged_button(icon_name: &str, count: &str, tooltip: &str) -> (gtk4::Button, gtk4::Label) {
    // A. The Base Icon
    let icon = gtk4::Image::builder()
        .icon_name(icon_name)
        .pixel_size(24)
        .build();
        
    // B. The Badge (Red Circle with Number)
    let badge = gtk4::Label::builder()
        .label(count)
        .css_classes(vec!["badge".to_string()]) // We will add CSS for this
        .halign(gtk4::Align::End)   // Top Right
        .valign(gtk4::Align::Start) 
        .visible(count != "0")      // Hide if 0 updates
        .build();

    // C. The Overlay (Stack them)
    let overlay = gtk4::Overlay::builder()
        .child(&icon) // Bottom layer
        .build();
    overlay.add_overlay(&badge); // Top layer

    // D. The Button containing the Overlay
    let btn = gtk4::Button::builder()
        .child(&overlay)
        .css_classes(vec!["circular-btn".to_string()])
        .height_request(30)
        .tooltip_text(tooltip)
        .build();
    (btn, badge)
}

// Helper to generate the Calendar Grid for a specific Month/Year
pub fn build_calendar_grid(year: i32, month: u32) -> gtk4::Grid {
    let grid = gtk4::Grid::builder()
        .column_spacing(5)
        .row_spacing(5)
        .hexpand(true)
        .vexpand(true)
        .halign(gtk4::Align::Fill)
        .valign(gtk4::Align::Fill)
        .column_homogeneous(true)
        .row_homogeneous(true)
        .build();

    // 1. HEADERS (Sun, Mon, Tue...)
    let days = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
    for (i, day) in days.iter().enumerate() {
        let label = gtk4::Label::builder()
            .label(*day)
            .css_classes(vec!["calendar-header".to_string()])
            .hexpand(true)
            .build();
        grid.attach(&label, i as i32, 0, 1, 1); // Row 0
    }

    // 2. MATH: Figure out padding and length
    // NaiveDate::from_ymd_opt is safe (handles invalid dates)
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    
    // num_days_from_sunday() gives 0 for Sun, 1 for Mon...
    // This tells us how many empty slots to put before the 1st.
    let start_offset = first_day.weekday().num_days_from_sunday(); 
    
    // Calculate days in month (The tricky way in Rust without a helper lib for it)
    // We get the first day of the NEXT month and subtract 1 day.
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let next_first = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
    let days_in_month = next_first.signed_duration_since(first_day).num_days();

    // 3. DRAW DAYS
    let mut col = start_offset as i32;
    let mut row = 1; // Start at Row 1 (Row 0 is headers)

    let today = Local::now().date_naive();

    for day_num in 1..=days_in_month {
        // Create the Button content (Number + Dot)
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        vbox.set_valign(gtk4::Align::Center);
        
        let num_label = gtk4::Label::builder()
            .label(day_num.to_string())
            .css_classes(vec!["calendar-day-num".to_string()])
            .build();
        
        // The "Red Dot" (Hidden by default)
        // LOGIC PLACEHOLDER: If day is divisible by 5, show dot (Demo only!)
        let has_appointment = day_num % 5 == 0; 
        
        let dot_label = gtk4::Label::builder()
            .label("•")
            .css_classes(vec!["calendar-dot".to_string()])
            .visible(has_appointment) // <--- Logic hooks here later
            .build();

        vbox.append(&num_label);
        vbox.append(&dot_label);

        let btn = gtk4::Button::builder()
            .child(&vbox)
            .css_classes(vec!["calendar-day-btn".to_string()])
            .hexpand(true)
            .vexpand(true)
            .valign(gtk4::Align::Fill)
            .build();

        // Highlight Today
        if today.year() == year && today.month() == month && today.day() == day_num as u32 {
            btn.add_css_class("today");
        }
        
        // Click Logic (TUI Link)
        btn.connect_clicked(move |_| {
            println!("Clicked Date: {}/{}/{}", year, month, day_num);
            let cmd = format!("ghostty --title=calendar-tui -e $HOME/.cargo/bin/cal-tui --date {}-{}-{}", year, month, day_num);
            run_cmd(&cmd);
        });

        grid.attach(&btn, col, row, 1, 1);

        // Advance Grid Cursor
        col += 1;
        if col > 6 {
            col = 0;
            row += 1;
        }
    }

    grid
}

// ROW 3 & 4 Sliders
pub fn make_slider_row(icon_name: &str) -> (gtk4::Box, gtk4::Scale) {
    let box_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    let icon = gtk4::Image::builder()
        .icon_name(icon_name)
        .pixel_size(20)
        .build();
    let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
    scale.set_hexpand(true);
    scale.set_draw_value(false);
    box_row.append(&icon);
    box_row.append(&scale);
    (box_row, scale)
}

pub fn run_cmd(cmd: &str) {
    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn();
}
