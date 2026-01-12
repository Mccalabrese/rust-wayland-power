use chrono::{NaiveDate, Utc};
use crate::engine::CalendarEngine;

pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    pub engine: CalendarEngine,
    pub current_date: NaiveDate, // The day selected in the UI
    pub should_quit: bool,
    pub input_mode: inputMode,
    pub input_buffer: String,
    pub cursor_position usize,
}

impl App {
    pub fn new() -> Self {
        // Load existing data or start fresh
        let engine = CalendarEngine::load_from_file("calendar_data.json");
        
        Self {
            engine,
            current_date: Utc::now().date_naive(), // Start on Today
            should_quit: false,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            cursor_position: 0,
        }
    }

    pub fn on_tick(&mut self) {
        // This is where we would handle auto-saving or background tasks later
    }

    pub fn quit(&mut self) {
        // Save on exit
        if let Err(e) = self.engine.save_to_file("calendar_data.json") {
            eprintln!("Failed to save on quit: {}", e);
        }
        self.should_quit = true;
    }
}
