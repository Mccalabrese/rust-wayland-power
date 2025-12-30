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
    let top_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    top_box.add_css_class("zone");

    top_box.set_margin_top(10);
    top_box.set_margin_bottom(10);
    top_box.set_margin_start(10);
    top_box.set_margin_end(10);

    fn make_icon_button(icon_name: &str) -> gtk4::Button {
        let icon = gtk4::Image::builder()
            .icon_name(icon_name)
            .pixel_size(24)
            .build();
        let btn = gtk4::Button::builder()
            .child(&icon)
            .css_classes(vec!["circular-btn".to_string()])
            .width_request(50)
            .height_request(50)
            .build();
        btn
    }

    // ---- ROW 1 Session Controls ----
    let row_session = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    row_session.set_halign(gtk4::Align::End);

    let btn_lock = make_icon_button("system-lock-screen-symbolic");
    let btn_logout = make_icon_button("system-log-out-symbolic");
    let btn_power = make_icon_button("system-shutdown-symbolic");

    row_session.append(&btn_lock);
    row_session.append(&btn_logout);
    row_session.append(&btn_power);

    //---- ROW 2 Toggles ----
    let row_toggles = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    row_toggles.set_halign(gtk4::Align::Center);

    let btn_wifi = make_icon_button("network-wireless-symbolic");
    let btn_bt = make_icon_button("bluetooth-active-symbolic");
    let btn_air = make_icon_button("airplane-mode-symbolic");
    let btn_dnd = make_icon_button("notifications-disabled-symbolic");
    let btn_mute = make_icon_button("audio-volume-muted-symbolic");

    row_toggles.append(&btn_wifi);
    row_toggles.append(&btn_bt);
    row_toggles.append(&btn_air);
    row_toggles.append(&btn_dnd);
    row_toggles.append(&btn_mute);

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
        margin: 10px;
        background-color: rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        }
        .circular-btn {
        border-radius: 99px;
        background-color: rgba(255, 255, 255, 0.1);
        color: white;
        border: none;
        box-shadow: none;
        }
        .circular-btn:hover {
            background-color: rgba(255, 255, 255, 0.3);
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
