use gtk4::prelude::*;
use gtk4::{gdk, Application, ApplicationWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

fn build_ui(app: &Application) {
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
    //2. Set the layer to Overlay
    window.set_layer(Layer::Overlay);
    //3. Anchor it to the Right, Top, and Bottom
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);

    window.set_width_request(final_width);

    load_css();
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);

    main_box.set_margin_top(10);
    main_box.set_margin_bottom(10);
    main_box.set_margin_start(10);
    main_box.set_margin_end(10);

    //Top Zone - Quick Toggles
    let top_box = gtk4::Box::new(gtk4::Orientation::Vertical, 15);
    top_box.add_css_class("zone");


    fn make_squared_button(icon_name: &str, tooltip: &str) -> gtk4::Button {
        let icon = gtk4::Image::builder()
            .icon_name(icon_name)
            .pixel_size(20)
            .build();
        let btn = gtk4::Button::builder()
            .child(&icon)
            .css_classes(vec!["squared-btn".to_string()])
            .height_request(20)
            .tooltip_text(tooltip)
            .build();
        btn
    }
    // 2. NEW: The Badged Button Helper (For Updates)
    fn make_badged_button(icon_name: &str, count: &str, tooltip: &str) -> gtk4::Button {
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
        gtk4::Button::builder()
            .child(&overlay)
            .css_classes(vec!["circular-btn".to_string()])
            .width_request(30)
            .height_request(30)
            .tooltip_text(tooltip)
            .build()
    }
    // ---- ROW 1 Session Controls ----
    let row_session = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    row_session.set_homogeneous(true);
    
    let btn_idle = make_squared_button("view-conceal-symbolic", "Idle Inhibit");
    let btn_suspend = make_squared_button("system-suspend-symbolic", "Suspend");
    let btn_lock = make_squared_button("system-lock-screen-symbolic", "Lock Screen");
    let btn_logout = make_squared_button("system-log-out-symbolic", "Logout");
    let btn_restart = make_squared_button("system-reboot-symbolic", "Reboot");
    let btn_power = make_squared_button("system-shutdown-symbolic", "Power Off");
    
    row_session.append(&btn_idle);
    row_session.append(&btn_suspend);
    row_session.append(&btn_lock);
    row_session.append(&btn_logout);
    row_session.append(&btn_restart);
    row_session.append(&btn_power);

    //---- ROW 2 Toggles ----
    
    fn make_icon_button(icon_name: &str, tooltip: &str) -> gtk4::Button {
        // Create the image part first, so we can control the size
        let icon = gtk4::Image::builder()
            .icon_name(icon_name)
            .pixel_size(24) // Now this works!
            .build();

        // Create the button and put the icon inside it
        let btn = gtk4::Button::builder()
            .child(&icon) // Use .child() instead of .icon_name()
            .css_classes(vec!["circular-btn".to_string()]) // Fix string types here too
            .height_request(30)
            .tooltip_text(tooltip)
            .build();
            
        btn
    }

    let row_toggles = gtk4::Box::new(gtk4::Orientation::Horizontal, 15);
    row_toggles.set_homogeneous(true);

    let btn_radio = make_icon_button("multimedia-player-symbolic", "Internet Radio");
    let btn_update = make_badged_button("software-update-available-symbolic", "14", "Update System");
    let btn_air = make_icon_button("airplane-mode-symbolic", "Airplane Mode");
    let btn_dns = make_icon_button("weather-overcast-symbolic", "Cloudflare DNS");
    let btn_mute = make_icon_button("audio-volume-muted-symbolic", "Mute Audio");
    let btn_wall = make_icon_button("image-x-generic-symbolic", "Change Wallpaper");
    let btn_hint = make_icon_button("emoji-objects-symbolic", "Show Keyhints");
    
    btn_idle.add_css_class("active");
    btn_dns.add_css_class("active");

    row_toggles.append(&btn_radio);
    row_toggles.append(&btn_update);
    row_toggles.append(&btn_air);
    row_toggles.append(&btn_dns);
    row_toggles.append(&btn_mute);
    row_toggles.append(&btn_wall);
    row_toggles.append(&btn_hint);

    // ROW 3 & 4 Sliders
    fn make_slider_row(icon_name: &str) -> gtk4::Box {
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
        box_row
    }
    let slider_brightness = make_slider_row("display-brightness-symbolic");
    let slider_volume = make_slider_row("audio-volume-high-symbolic");

    top_box.append(&row_session);
    top_box.append(&row_toggles);
    top_box.append(&slider_brightness);
    top_box.append(&slider_volume);

    //Middle Zone - Notifications
    let middle_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    middle_box.add_css_class("zone");
    middle_box.set_vexpand(true);

    //Bottom Zone - Calender
    let bottom_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    bottom_box.add_css_class("zone");
    bottom_box.set_height_request(calendar_height);

    main_box.append(&top_box);
    main_box.append(&middle_box);
    main_box.append(&bottom_box);

    window.set_child(Some(&main_box));
    window.present();
}
fn main() {
    let app = Application::builder()
        .application_id("com.student.sidebar")
        .build();

    //Activate signal
    app.connect_activate(build_ui);

    app.run();
}

fn load_css() {
    //create a CSS provider
    let provider = gtk4::CssProvider::new();
    //Css logic
    provider.load_from_data("
        window {
        background-color: rgba(30, 30, 46, 0.95);
        }
        .zone {
        padding: 12px;
        background-color: rgba(255, 255, 255, 0.08);
        border-radius: 12px;
        }
        .circular-btn {
        border-radius: 99px;
        background-color: rgba(255, 255, 255, 0.1);
        color: white;
        border: none;
        box-shadow: none;
        background-image: none;
        }
        .squared-btn {
        border-radius: 8px;
        background-color: rgba(255, 255, 255, 0.1);
        color: white;
        border: none;
        box-shadow: none;
        padding: 0px;
        background-image: none;
        }
        .circular-btn:hover, .squared-btn:hover {
            background-color: rgba(255, 255, 255, 0.2);
        }
        .circular-btn.active, .squared-btn.active {
            background-color: #3584e4;
            color: white;
            background-image: none;
        }
        .circular-btn.active:hover, .squared-btn.active:hover {
            background-color: #1c71d8;
        }
        .icon-text {
            font-size: 16px;
            font-weight: bold;
        }
        .badge {
            background-color: #ff5555; /* Red */
            color: white;
            border-radius: 99px;
            min-width: 14px;
            min-height: 14px;
            font-size: 10px;
            font-weight: bold;
            padding-left: 3px;
            padding-right: 3px;
            margin-top: -5px;  /* Nudge it up */
            margin-right: -5px; /* Nudge it right */
        }
    ");
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
