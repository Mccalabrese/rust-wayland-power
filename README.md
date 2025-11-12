# Arch-multi-session-dot-files

**Nothing is ready to upload yet Im currently editing and cleaning for API's or secrets and using the readme here to keep track things**
I run a 4 session arch setup with Niri, gnome, sway, and hyprland. Here are my dots
🚀 Dotfiles Setup Guide (Running List)

This guide covers the dependencies and manual steps required to set up this Arch Linux environment.

1. System Dependencies (pacman)

These packages must be installed from the official Arch repositories.
Bash

# Core window managers & display
sudo pacman -S sway hyprland niri gnome

# Core applications & utilities
sudo pacman -S waybar hyprlock swayidle wofi
sudo pacman -S polkit-gnome nm-applet udiskie thunar
sudo pacman -S ghostty

# Audio
sudo pacman -S pulseaudio # (or pipewire-pulse)

# Location services
sudo pacman -S geoclue

# Update checking
sudo pacman -S pacman-contrib # (Provides 'checkupdates')

# Build tools for Rust
sudo pacman -S openssl pkg-config

2. API Key Setup (Manual Steps)

This setup requires two external API keys.

    Google Geolocation API:

        Create a project in the Google Cloud Console and enable the "Geolocation API".

        Generate an API key.

        Edit /etc/geoclue/geoclue.conf (requires sudo).

        Add your key to the [wifi-scan] section: url=https://www.googleapis.com/geolocation/v1/geolocate?key=YOUR_GOOGLE_KEY_HERE

        Restart the service: sudo systemctl restart geoclue.service.

    OpenWeatherMap (OWM) API:

        Create a free account at OpenWeatherMap and generate an API key.

        This key will be placed directly into your Waybar config files in Step 5.

3. Install Rust Toolchain

We use rustup to manage the Rust compiler.
Bash

# Install rustup
sudo pacman -S rustup
# Run the installer as your normal user
rustup-init

4. Build & Install Custom Rust Apps

These scripts are built from source and installed to your user's local bin directory.
Bash

# First, add cargo to your $PATH
# (This step will be done by your .zshrc/.zshenv)
export PATH="$HOME/.cargo/bin:$PATH"

# Go to your sysScripts or dotfiles repo...
cd /path/to/your/dotfiles

# Install the weather app
cd waybar-weather
cargo install --path .

# Install the update-check app
cd ../update-check
cargo install --path .

# Install the sway-workspace app
cd ../sway-workspace
cargo install --path .

# Install the waybar-switcher app
cd ../waybar-switcher
cargo install --path .

5. Post-Install Configuration

Your configuration files need to be "universal" by using $HOME and pointing to your OWM API key.

    Session Startup Scripts (~/.config/hypr/hyprland.conf, ~/.config/sway/config, ~/.config/niri/niri.conf):

        Find the exec-once or spawn-at-startup line for waybar-switcher.

        Ensure it uses the absolute path with $HOME: exec-once = $HOME/.cargo/bin/waybar-switcher

    Waybar Configs (~/.config/waybar/hyprConfig.jsonc, etc.):

        Find the custom/weather module.

        Change its exec line to use $HOME and include your API key: "exec": "OWM_API_KEY=YOUR_OWM_KEY_HERE $HOME/.cargo/bin/waybar-weather",

        Find the custom/updater module.

        Change its exec line to use $HOME: "exec": "$HOME/.cargo/bin/update-check",

        (Note: The sway-workspace script is called by Waybar, but your swayConfig.jsonc likely doesn't exist yet! We'll need to create it and add the module.)
