// ═══════════════════════════════════════════════════════════════════════════════
// 📦 app.rs - Application Logic
// ═══════════════════════════════════════════════════════════════════════════════
// This module contains the main application logic and event handling.
// Features:
// - Event loop management
// - Keyboard input handling
// - Integration of all components
// ═══════════════════════════════════════════════════════════════════════════════

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::csv_loader::pick_and_load_csv;
use crate::detectors::quick_detect;
use crate::serial_reader::SerialReader;
use crate::state::SharedState;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Application Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Tick rate for the event loop in milliseconds
const TICK_RATE_MS: u64 = 50;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Application Structure
// ═══════════════════════════════════════════════════════════════════════════════

/// Main application structure
pub struct App {
    /// Shared application state
    state: SharedState,
    
    /// Serial reader instance
    serial_reader: Option<SerialReader>,
}

impl App {
    /// Create a new application instance
    pub fn new(state: SharedState) -> Self {
        Self {
            state,
            serial_reader: None,
        }
    }

    /// Handle keyboard and other events
    ///
    /// Returns true if should quit
    pub fn handle_events(&mut self) -> Result<bool, String> {
        // Poll for events with timeout
        if event::poll(Duration::from_millis(TICK_RATE_MS))
            .map_err(|e| format!("Event poll error: {}", e))?
        {
            if let Event::Key(key) = event::read().map_err(|e| format!("Event read error: {}", e))? {
                // Only handle key press events
                if key.kind == KeyEventKind::Press {
                    return self.handle_key(key.code);
                }
            }
        }

        Ok(false)
    }

    /// Handle a single key press
    fn handle_key(&mut self, key: KeyCode) -> Result<bool, String> {
        match key {
            // Q - Quit
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                return Ok(true);
            }

            // S - Start Serial
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Stop playback mode first
                {
                    let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                    state_guard.stop_playback();
                }
                self.start_serial()?;
            }

            // X - Stop Serial
            KeyCode::Char('x') | KeyCode::Char('X') => {
                self.stop_serial();
                // Also stop playback
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                state_guard.stop_playback();
            }

            // L - Load CSV
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.load_csv()?;
            }

            // Space - Play/Pause playback
            KeyCode::Char(' ') => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                state_guard.toggle_playback();
                let status = if state_guard.playback_playing { "▶️ Playing" } else { "⏸️ Paused" };
                state_guard.status_message = format!("{} - {:.1}s / {:.1}s", 
                    status,
                    state_guard.get_current_playback_second(),
                    state_guard.playback_duration_secs
                );
            }

            // Left Arrow - Seek backward 5 seconds
            KeyCode::Left => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                if state_guard.playback_mode {
                    state_guard.seek_by_seconds(-5.0);
                    state_guard.status_message = format!("⏪ Seek: {:.1}s / {:.1}s",
                        state_guard.get_current_playback_second(),
                        state_guard.playback_duration_secs
                    );
                }
            }

            // Right Arrow - Seek forward 5 seconds
            KeyCode::Right => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                if state_guard.playback_mode {
                    state_guard.seek_by_seconds(5.0);
                    state_guard.status_message = format!("⏩ Seek: {:.1}s / {:.1}s",
                        state_guard.get_current_playback_second(),
                        state_guard.playback_duration_secs
                    );
                }
            }

            // Up Arrow - Seek backward 30 seconds
            KeyCode::Up => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                if state_guard.playback_mode {
                    state_guard.seek_by_seconds(-30.0);
                    state_guard.status_message = format!("⏪⏪ Seek: {:.1}s / {:.1}s",
                        state_guard.get_current_playback_second(),
                        state_guard.playback_duration_secs
                    );
                }
            }

            // Down Arrow - Seek forward 30 seconds
            KeyCode::Down => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                if state_guard.playback_mode {
                    state_guard.seek_by_seconds(30.0);
                    state_guard.status_message = format!("⏩⏩ Seek: {:.1}s / {:.1}s",
                        state_guard.get_current_playback_second(),
                        state_guard.playback_duration_secs
                    );
                }
            }

            // Home - Go to start
            KeyCode::Home => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                if state_guard.playback_mode {
                    state_guard.seek_to_second(0.0);
                    state_guard.status_message = "⏮️ Start".to_string();
                }
            }

            // End - Go to end
            KeyCode::End => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                if state_guard.playback_mode {
                    let duration = state_guard.playback_duration_secs;
                    state_guard.seek_to_second(duration);
                    state_guard.status_message = "⏭️ End".to_string();
                }
            }

            // R - Restart playback
            KeyCode::Char('r') | KeyCode::Char('R') => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                if state_guard.playback_mode {
                    state_guard.seek_to_second(0.0);
                    state_guard.playback_playing = true;
                    state_guard.status_message = "🔄 Restarted".to_string();
                }
            }

            // B - Back to Live Mode
            KeyCode::Char('b') | KeyCode::Char('B') => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                if state_guard.playback_mode {
                    // Exit playback mode
                    state_guard.playback_mode = false;
                    state_guard.playback_playing = false;
                    state_guard.loaded_frames.clear();
                    state_guard.playback_position = 0;
                    state_guard.status_message = "📡 Live Mode - Press C to connect".to_string();
                }
            }

            // Escape - Quit
            KeyCode::Esc => {
                return Ok(true);
            }

            _ => {}
        }

        Ok(false)
    }

    /// Start the serial reader
    fn start_serial(&mut self) -> Result<(), String> {
        // Stop existing reader if any
        self.stop_serial();

        // Create and start new reader
        let mut reader = SerialReader::new(self.state.clone());
        
        if let Err(e) = reader.start() {
            let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
            state_guard.status_message = format!("❌ {}", e);
            return Err(e);
        }

        self.serial_reader = Some(reader);
        Ok(())
    }

    /// Stop the serial reader
    fn stop_serial(&mut self) {
        if let Some(ref mut reader) = self.serial_reader {
            reader.stop();
        }
        self.serial_reader = None;
    }

    /// Load CSV file
    fn load_csv(&mut self) -> Result<(), String> {
        // Stop serial reader if running
        self.stop_serial();

        // Show loading message
        {
            let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
            state_guard.status_message = "📂 Opening file dialog...".to_string();
        }

        // Pick and load CSV file
        match pick_and_load_csv(&self.state) {
            Ok(count) => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                state_guard.status_message = format!("✅ Loaded {} frames from CSV", count);
            }
            Err(e) => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                state_guard.status_message = format!("❌ {}", e);
            }
        }

        Ok(())
    }

    /// Run detection algorithms on current frames
    pub fn run_detectors(&mut self) -> Result<(), String> {
        let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
        
        // Run detectors on all frames
        let results = quick_detect(&state_guard.frames);
        
        // Update detection results
        state_guard.detections = results;
        
        // Update history for charts
        state_guard.update_detection_history();

        Ok(())
    }

    /// Cleanup resources before exit
    fn cleanup(&mut self) {
        // Stop serial reader
        self.stop_serial();

        // Flush CSV logger if exists
        if let Ok(mut state_guard) = self.state.lock() {
            if let Some(ref mut logger) = state_guard.csv_logger {
                let _ = logger.flush();
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.cleanup();
    }
}
