use gtk4::prelude::*;
use gtk4::Application;

mod ui;
mod media;
mod style;
mod helpers;
mod sysinfo;

fn main() {
    unsafe {
        std::env::set_var("GTK_A11Y", "none");
        std::env::set_var("GTK_USE_PORTAL", "0");
        std::env::set_var("GSK_RENDERER", "cairo"); 
    }

    let app = Application::builder()
        .build();

    app.connect_activate(ui::build_ui);

    app.run();
}
