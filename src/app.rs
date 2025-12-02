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
// 🔹 Application Configuration / إعدادات التطبيق
// ═══════════════════════════════════════════════════════════════════════════════

/// Tick rate for the event loop in milliseconds
/// معدل التحديث لحلقة الأحداث بالميلي ثانية
const TICK_RATE_MS: u64 = 50;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Application Structure / هيكل التطبيق
// ═══════════════════════════════════════════════════════════════════════════════

/// Main application structure
/// هيكل التطبيق الرئيسي
pub struct App {
    /// Shared application state / حالة التطبيق المشتركة
    state: SharedState,
    
    /// Serial reader instance / مثيل قارئ التسلسل
    serial_reader: Option<SerialReader>,
}

impl App {
    /// Create a new application instance
    /// إنشاء مثيل تطبيق جديد
    pub fn new(state: SharedState) -> Self {
        Self {
            state,
            serial_reader: None,
        }
    }

    /// Handle keyboard and other events
    /// معالجة لوحة المفاتيح والأحداث الأخرى
    /// 
    /// Returns true if should quit / يرجع true إذا يجب الخروج
    pub fn handle_events(&mut self) -> Result<bool, String> {
        // Poll for events with timeout / استطلاع الأحداث مع مهلة
        if event::poll(Duration::from_millis(TICK_RATE_MS))
            .map_err(|e| format!("Event poll error: {}", e))?
        {
            if let Event::Key(key) = event::read().map_err(|e| format!("Event read error: {}", e))? {
                // Only handle key press events / معالجة أحداث الضغط على المفاتيح فقط
                if key.kind == KeyEventKind::Press {
                    return self.handle_key(key.code);
                }
            }
        }

        Ok(false)
    }

    /// Handle a single key press
    /// معالجة ضغطة مفتاح واحدة
    fn handle_key(&mut self, key: KeyCode) -> Result<bool, String> {
        match key {
            // Q - Quit / الخروج
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                return Ok(true);
            }

            // S - Start Serial / بدء التسلسل
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Stop playback mode first / إيقاف وضع التشغيل أولاً
                {
                    let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                    state_guard.stop_playback();
                }
                self.start_serial()?;
            }

            // X - Stop Serial / إيقاف التسلسل
            KeyCode::Char('x') | KeyCode::Char('X') => {
                self.stop_serial();
                // Also stop playback / إيقاف التشغيل أيضاً
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                state_guard.stop_playback();
            }

            // L - Load CSV / تحميل CSV
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.load_csv()?;
            }

            // Space - Play/Pause playback / تشغيل/إيقاف مؤقت
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

            // Left Arrow - Seek backward 5 seconds / ترجيع 5 ثواني
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

            // Right Arrow - Seek forward 5 seconds / تقديم 5 ثواني
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

            // Up Arrow - Seek backward 30 seconds / ترجيع 30 ثانية
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

            // Down Arrow - Seek forward 30 seconds / تقديم 30 ثانية
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

            // Home - Go to start / الذهاب للبداية
            KeyCode::Home => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                if state_guard.playback_mode {
                    state_guard.seek_to_second(0.0);
                    state_guard.status_message = "⏮️ Start".to_string();
                }
            }

            // End - Go to end / الذهاب للنهاية
            KeyCode::End => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                if state_guard.playback_mode {
                    let duration = state_guard.playback_duration_secs;
                    state_guard.seek_to_second(duration);
                    state_guard.status_message = "⏭️ End".to_string();
                }
            }

            // R - Restart playback / إعادة التشغيل
            KeyCode::Char('r') | KeyCode::Char('R') => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                if state_guard.playback_mode {
                    state_guard.seek_to_second(0.0);
                    state_guard.playback_playing = true;
                    state_guard.status_message = "🔄 Restarted".to_string();
                }
            }

            // B - Back to Live Mode / العودة للبث المباشر
            KeyCode::Char('b') | KeyCode::Char('B') => {
                let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
                if state_guard.playback_mode {
                    // Exit playback mode / الخروج من وضع التشغيل
                    state_guard.playback_mode = false;
                    state_guard.playback_playing = false;
                    state_guard.loaded_frames.clear();
                    state_guard.playback_position = 0;
                    state_guard.status_message = "📡 Live Mode - Press C to connect".to_string();
                }
            }

            // Escape - Quit / الخروج
            KeyCode::Esc => {
                return Ok(true);
            }

            _ => {}
        }

        Ok(false)
    }

    /// Start the serial reader
    /// بدء قارئ التسلسل
    fn start_serial(&mut self) -> Result<(), String> {
        // Stop existing reader if any / إيقاف القارئ الموجود إذا كان موجوداً
        self.stop_serial();

        // Create and start new reader / إنشاء وبدء قارئ جديد
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
    /// إيقاف قارئ التسلسل
    fn stop_serial(&mut self) {
        if let Some(ref mut reader) = self.serial_reader {
            reader.stop();
        }
        self.serial_reader = None;
    }

    /// Load CSV file
    /// تحميل ملف CSV
    fn load_csv(&mut self) -> Result<(), String> {
        // Stop serial reader if running / إيقاف قارئ التسلسل إذا كان يعمل
        self.stop_serial();

        // Show loading message / عرض رسالة التحميل
        {
            let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
            state_guard.status_message = "📂 Opening file dialog...".to_string();
        }

        // Pick and load CSV file / اختيار وتحميل ملف CSV
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
    /// تشغيل خوارزميات الكشف على الإطارات الحالية
    pub fn run_detectors(&mut self) -> Result<(), String> {
        let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
        
        // Run detectors on all frames / تشغيل الكاشفات على جميع الإطارات
        let results = quick_detect(&state_guard.frames);
        
        // Update detection results / تحديث نتائج الكشف
        state_guard.detections = results;
        
        // Update history for charts / تحديث التاريخ للرسوم البيانية
        state_guard.update_detection_history();

        Ok(())
    }

    /// Cleanup resources before exit
    /// تنظيف الموارد قبل الخروج
    fn cleanup(&mut self) {
        // Stop serial reader / إيقاف قارئ التسلسل
        self.stop_serial();

        // Flush CSV logger if exists / تفريغ مسجل CSV إذا كان موجوداً
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
