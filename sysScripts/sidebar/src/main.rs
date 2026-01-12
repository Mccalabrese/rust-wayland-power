//! Rust Sidebar (Entry Point)
//! 
//! A native GTK4 LayerShell sidebar designed for Wayland compositors (Niri, Sway, Hyprland).
//! 
//! Architecture:
//! - **main.rs**: Entry point, environment setup, and module registration.
//! - **ui.rs**: Main widget layout, window creation, and event wiring.
//! - **style.rs**: CSS styling and theming (Catppuccin/Glassmorphism).
//! - **helpers.rs**: Shared utilities (command execution, button factories).
//! - **media.rs**: Dynamic "Now Playing" widget (Playerctl integration).
//! - **sysinfo.rs**: System status widget (Static snapshot).

use gtk4::prelude::*;
use gtk4::Application;

// --- Module Registration ---
mod ui;      // The layout builder
mod media;   // Media player logic
mod style;   // CSS provider
mod helpers; // Utility functions
mod sysinfo; // System fetch widget

fn main() {
    // 1. Environment Configuration
    // We set these variables BEFORE initializing GTK to ensure they take effect.
    // This block is marked 'unsafe' because modifying environment variables 
    // is not thread-safe, but since we are the only thread at startup, it's fine.
    unsafe {
        // PERF: Disable the Accessibility Bus. 
        // In minimal WMs, this service is often missing, causing GTK apps to 
        // hang for 25s at startup while waiting for a timeout.
        std::env::set_var("GTK_A11Y", "none");
        // COMPAT: Force native file choosers (avoids portal issues in some setups).
        std::env::set_var("GTK_USE_PORTAL", "0");
        // STABILITY: Force the 'Cairo' renderer instead of OpenGL/Vulkan.
        // On some NVIDIA cards or older iGPUs, the GL renderer causes 
        // transparent windows (like this sidebar) to flicker or turn black.
        // Cairo is CPU-based, slightly slower, but rock-solid for 2D UI.
        std::env::set_var("GSK_RENDERER", "cairo"); 
    }

    // 2. Application Setup
    // We use the standard GTK4 Application builder.
    // We do NOT set an application ID (like "com.example.sidebar") to avoid 
    // DBus uniqueness checks, allowing multiple instances if needed (though rare).
    let app = Application::builder()
        .build();

    // 3. Connect the UI
    // When the app starts ('activate'), run the build_ui function from ui.rs
    app.connect_activate(ui::build_ui);

    // 4. Run Event Loop
    // This blocks the main thread until the window is closed.
    app.run();
}
