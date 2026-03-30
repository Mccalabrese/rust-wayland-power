use std::fs;
use std::thread;
use std::time::Duration;
use std::process::Command;

fn get_battery_name() -> Option<String> {
    let battery_path = fs::read_dir("/sys/class/power_supply/").ok()?;
        for path in battery_path {
            if let Ok(entry) = path {
                let battery_file = entry.file_name().into_string().unwrap_or_default();
                if battery_file.starts_with("BAT") {
                    return Some(battery_file);
                }
            }
        }
    None
}

fn main() {
    let battery_detection = match get_battery_name() {
        Some(name) => name,
        None => {
            return;
        }
    };
    let battery_path = format!("/sys/class/power_supply/{}/capacity", battery_detection); // path to check current battery capacity
    let status_path = format!("/sys/class/power_supply/{}/status", battery_detection); // path to check the status (discharging, charging, etc)

    // These are for the popup warnings to ensure that they will show up no matter what
    let mut warning_10 = false; 
    let mut warning_5 = false;
    let mut shutting_down = false;

    loop {
        thread::sleep(Duration::from_secs(30)); // This is for your computer to run a check every 30 seconds to keep an eye on the battery

        let capacity_string = match fs::read_to_string(&battery_path) {
            Ok(battery_path_detection) => battery_path_detection,
            Err(_) => {
                continue;
            }
        };
        let status_string = match fs::read_to_string(&status_path) {
            Ok(status_path_detection) => status_path_detection,
            Err(_) => {
                continue;
            }
        };

        let capacity_int = match capacity_string.trim().parse::<u8>() {
            Ok(capacity) => capacity,
            Err(_) => {
                continue;
            }
        };

        let status  = status_string.trim();

            if capacity_int <= 10 && status == "Discharging" && !warning_10 && !shutting_down { // battery warning at 10%
                let _ = Command::new("/usr/bin/notify-send").arg("Battery Warning 10%").arg("Shuts down at 3%").spawn();
                warning_10 = true;
            } 
            if  capacity_int <= 5 && status == "Discharging" && !warning_5 && !shutting_down { // battery warning 5%
                let _ = Command::new("/usr/bin/notify-send").arg("Battery Warning 5%").arg("Shuts down at 3%\nSAVE WORK NOW").spawn();
                warning_5 = true;
            } 
            if status != "Discharging" { // prevents losing the warnings if you replug and let the computer drain again
                warning_10 = false;
                warning_5 = false;
            }

            // Here is where the magic happens
            if capacity_int <= 3 && status == "Discharging" && !shutting_down { 
                shutting_down = true;
                let _ = Command::new("/usr/bin/systemctl").arg("poweroff").spawn();
             } 
    } 
}
