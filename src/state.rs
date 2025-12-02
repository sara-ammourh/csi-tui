// ═══════════════════════════════════════════════════════════════════════════════
// 📦 state.rs - Application State Management
// ═══════════════════════════════════════════════════════════════════════════════
// This module defines the core data structures for CSI frames and application state.
// Uses Arc<Mutex> for thread-safe sharing between serial reader and TUI threads.
// ═══════════════════════════════════════════════════════════════════════════════

use std::sync::{Arc, Mutex};
use crate::csv_logger::CsvLogger;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 CSI Format Enum / نوع صيغة بيانات CSI
// ═══════════════════════════════════════════════════════════════════════════════

/// Represents the format of CSI data received from ESP32
/// يمثل صيغة بيانات CSI المستلمة من ESP32
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CsiFormat {
    /// Real and Imaginary pairs (r, i) / أزواج حقيقية وتخيلية
    RealImag,
    /// Amplitude only values / قيم السعة فقط
    AmplitudeOnly,
    /// Unknown format / صيغة غير معروفة
    Unknown,
}

impl Default for CsiFormat {
    fn default() -> Self {
        CsiFormat::Unknown
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 CSI Frame Structure / هيكل إطار CSI
// ═══════════════════════════════════════════════════════════════════════════════

/// Represents a single CSI frame captured from WiFi signal
/// يمثل إطار CSI واحد ملتقط من إشارة الواي فاي
#[derive(Debug, Clone)]
pub struct CsiFrame {
    /// Unix timestamp in milliseconds / الطابع الزمني بالميلي ثانية
    pub timestamp: i64,
    
    /// Calculated magnitudes for each subcarrier / السعات المحسوبة لكل ناقل فرعي
    /// mag = sqrt(real² + imag²) for RealImag format
    pub mags: Vec<f64>,
    
    /// Raw (real, imag) pairs from CSI data / الأزواج الخام (حقيقي، تخيلي)
    pub pairs: Vec<(i32, i32)>,
    
    /// The detected format of this frame / صيغة هذا الإطار المكتشفة
    #[allow(dead_code)]
    pub format: CsiFormat,
}

impl CsiFrame {
    /// Create a new CSI frame / إنشاء إطار CSI جديد
    pub fn new(timestamp: i64, mags: Vec<f64>, pairs: Vec<(i32, i32)>, format: CsiFormat) -> Self {
        Self {
            timestamp,
            mags,
            pairs,
            format,
        }
    }

    /// Get the number of subcarriers / الحصول على عدد الناقلات الفرعية
    pub fn subcarrier_count(&self) -> usize {
        self.mags.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Detection Results / نتائج الكشف
// ═══════════════════════════════════════════════════════════════════════════════

/// Holds the results of all detection algorithms
/// يحتوي على نتائج جميع خوارزميات الكشف
#[derive(Debug, Clone, Default)]
pub struct DetectionResults {
    /// Motion detected / تم كشف حركة
    pub motion_detected: bool,
    
    /// Human presence detected / تم كشف وجود بشري
    pub human_present: bool,
    
    /// Door state changed / تغيرت حالة الباب
    pub door_open: bool,
    
    /// Motion intensity value (0-100) / قيمة شدة الحركة
    pub motion_value: f64,
    
    /// Human presence value (0-100) / قيمة الوجود البشري
    pub presence_value: f64,
    
    /// Door change value (0-100) / قيمة تغير الباب
    pub door_value: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Application State / حالة التطبيق
// ═══════════════════════════════════════════════════════════════════════════════

/// Main application state shared between threads
/// حالة التطبيق الرئيسية المشتركة بين الخيوط
pub struct AppState {
    /// Is the serial receiver currently active? / هل المستقبل التسلسلي نشط حالياً؟
    pub receiver_active: bool,
    
    /// All CSI frames in memory (last 60 seconds) / جميع إطارات CSI في الذاكرة (آخر 60 ثانية)
    pub frames: Vec<CsiFrame>,
    
    /// Maximum number of subcarriers ever seen / أقصى عدد ناقلات فرعية تم رؤيته
    pub max_sc: usize,
    
    /// CSV logger instance (optional) / مثيل مسجل CSV (اختياري)
    pub csv_logger: Option<CsvLogger>,
    
    /// Current detection results / نتائج الكشف الحالية
    pub detections: DetectionResults,
    
    /// Status message to display / رسالة الحالة للعرض
    pub status_message: String,
    
    /// Serial port name / اسم المنفذ التسلسلي
    pub port_name: String,
    
    /// Should the application quit? / هل يجب إنهاء التطبيق؟
    pub should_quit: bool,
    
    /// History of motion values for chart / تاريخ قيم الحركة للرسم البياني
    pub motion_history: Vec<f64>,
    
    /// History of presence values for chart / تاريخ قيم الوجود للرسم البياني
    pub presence_history: Vec<f64>,
    
    /// History of door values for chart / تاريخ قيم الباب للرسم البياني
    pub door_history: Vec<f64>,
    
    // ═══════════════════════════════════════════════════════════════════════
    // 🎬 Playback Mode Fields / حقول وضع التشغيل
    // ═══════════════════════════════════════════════════════════════════════
    
    /// All loaded frames from CSV (for playback) / جميع الإطارات المحملة من CSV (للتشغيل)
    pub loaded_frames: Vec<CsiFrame>,
    
    /// Is playback mode active? / هل وضع التشغيل نشط؟
    pub playback_mode: bool,
    
    /// Is playback currently playing? / هل التشغيل جارٍ حالياً؟
    pub playback_playing: bool,
    
    /// Current playback position (frame index) / موقع التشغيل الحالي (فهرس الإطار)
    pub playback_position: usize,
    
    /// Total duration of loaded data in seconds / المدة الإجمالية للبيانات المحملة بالثواني
    pub playback_duration_secs: f64,
}

impl AppState {
    /// Create a new AppState with default values
    /// إنشاء حالة تطبيق جديدة بقيم افتراضية
    pub fn new() -> Self {
        Self {
            receiver_active: false,
            frames: Vec::new(),
            max_sc: 0,
            csv_logger: None,
            detections: DetectionResults::default(),
            status_message: "Press S to start serial, L to load CSV".to_string(),
            port_name: "COM3".to_string(),
            should_quit: false,
            motion_history: Vec::new(),
            presence_history: Vec::new(),
            door_history: Vec::new(),
            // Playback fields
            loaded_frames: Vec::new(),
            playback_mode: false,
            playback_playing: false,
            playback_position: 0,
            playback_duration_secs: 0.0,
        }
    }

    /// Add a new CSI frame and maintain 60-second window
    /// إضافة إطار CSI جديد والحفاظ على نافذة 60 ثانية
    pub fn push_frame(&mut self, frame: CsiFrame) {
        // Update max subcarrier count / تحديث أقصى عدد للناقلات الفرعية
        if frame.subcarrier_count() > self.max_sc {
            self.max_sc = frame.subcarrier_count();
        }

        // Add the frame / إضافة الإطار
        self.frames.push(frame);

        // Remove frames older than 60 seconds / حذف الإطارات الأقدم من 60 ثانية
        self.cleanup_old_frames();
    }

    /// Update detection history for charts
    /// تحديث تاريخ الكشف للرسوم البيانية
    pub fn update_detection_history(&mut self) {
        const MAX_HISTORY: usize = 100;
        
        // Add current values to history / إضافة القيم الحالية للتاريخ
        self.motion_history.push(self.detections.motion_value);
        self.presence_history.push(self.detections.presence_value);
        self.door_history.push(self.detections.door_value);
        
        // Keep only last MAX_HISTORY values / الاحتفاظ بآخر MAX_HISTORY قيمة فقط
        if self.motion_history.len() > MAX_HISTORY {
            self.motion_history.remove(0);
        }
        if self.presence_history.len() > MAX_HISTORY {
            self.presence_history.remove(0);
        }
        if self.door_history.len() > MAX_HISTORY {
            self.door_history.remove(0);
        }
    }

    /// Remove frames older than 60 seconds
    /// حذف الإطارات الأقدم من 60 ثانية
    fn cleanup_old_frames(&mut self) {
        let now = chrono::Utc::now().timestamp_millis();
        let cutoff = now - 60_000; // 60 seconds in milliseconds
        
        self.frames.retain(|f| f.timestamp > cutoff);
    }

    /// Get the last N frames for display
    /// الحصول على آخر N إطار للعرض
    pub fn get_last_frames(&self, count: usize) -> &[CsiFrame] {
        let len = self.frames.len();
        if len <= count {
            &self.frames
        } else {
            &self.frames[len - count..]
        }
    }

    /// Get total frame count
    /// الحصول على العدد الإجمالي للإطارات
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Clear all frames
    /// مسح جميع الإطارات
    pub fn clear_frames(&mut self) {
        self.frames.clear();
        self.max_sc = 0;
        self.motion_history.clear();
        self.presence_history.clear();
        self.door_history.clear();
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 🎬 Playback Control Methods / دوال التحكم بالتشغيل
    // ═══════════════════════════════════════════════════════════════════════

    /// Start playback mode with loaded frames
    /// بدء وضع التشغيل مع الإطارات المحملة
    pub fn start_playback(&mut self) {
        if self.loaded_frames.is_empty() {
            return;
        }
        
        self.playback_mode = true;
        self.playback_playing = true;
        self.playback_position = 0;
        
        // Calculate duration from timestamps
        // حساب المدة من الطوابع الزمنية
        if let (Some(first), Some(last)) = (self.loaded_frames.first(), self.loaded_frames.last()) {
            self.playback_duration_secs = (last.timestamp - first.timestamp) as f64 / 1000.0;
        }
        
        // Clear current display frames
        self.frames.clear();
        self.motion_history.clear();
        self.presence_history.clear();
        self.door_history.clear();
    }

    /// Toggle playback play/pause
    /// تبديل التشغيل/الإيقاف المؤقت
    pub fn toggle_playback(&mut self) {
        if self.playback_mode {
            self.playback_playing = !self.playback_playing;
        }
    }

    /// Stop playback and return to normal mode
    /// إيقاف التشغيل والعودة للوضع العادي
    pub fn stop_playback(&mut self) {
        self.playback_mode = false;
        self.playback_playing = false;
        self.playback_position = 0;
    }

    /// Seek to a specific second in playback
    /// الانتقال لثانية محددة في التشغيل
    pub fn seek_to_second(&mut self, second: f64) {
        if self.loaded_frames.is_empty() {
            return;
        }
        
        let first_ts = self.loaded_frames[0].timestamp;
        let target_ts = first_ts + (second * 1000.0) as i64;
        
        // Find the frame closest to target timestamp
        // البحث عن الإطار الأقرب للطابع الزمني المستهدف
        self.playback_position = self.loaded_frames
            .iter()
            .position(|f| f.timestamp >= target_ts)
            .unwrap_or(0);
        
        // Reset display frames from this position
        // إعادة تعيين إطارات العرض من هذا الموقع
        self.frames.clear();
        self.motion_history.clear();
        self.presence_history.clear();
        self.door_history.clear();
    }

    /// Seek forward/backward by seconds
    /// التقديم/الترجيع بالثواني
    pub fn seek_by_seconds(&mut self, delta: f64) {
        let current_sec = self.get_current_playback_second();
        let new_sec = (current_sec + delta).max(0.0).min(self.playback_duration_secs);
        self.seek_to_second(new_sec);
    }

    /// Get current playback position in seconds
    /// الحصول على موقع التشغيل الحالي بالثواني
    pub fn get_current_playback_second(&self) -> f64 {
        if self.loaded_frames.is_empty() || self.playback_position >= self.loaded_frames.len() {
            return 0.0;
        }
        
        let first_ts = self.loaded_frames[0].timestamp;
        let current_ts = self.loaded_frames[self.playback_position].timestamp;
        
        (current_ts - first_ts) as f64 / 1000.0
    }

    /// Advance playback by one frame
    /// تقديم التشغيل بإطار واحد
    pub fn advance_playback(&mut self) -> Option<CsiFrame> {
        if !self.playback_mode || !self.playback_playing {
            return None;
        }
        
        if self.playback_position >= self.loaded_frames.len() {
            // Reached end, loop back or stop
            // وصلنا للنهاية، إعادة من البداية أو إيقاف
            self.playback_position = 0;
            self.frames.clear();
            self.motion_history.clear();
            self.presence_history.clear();
            self.door_history.clear();
            return None;
        }
        
        let frame = self.loaded_frames[self.playback_position].clone();
        self.playback_position += 1;
        
        Some(frame)
    }

    /// Get playback progress as percentage (0.0 - 1.0)
    /// الحصول على تقدم التشغيل كنسبة مئوية
    pub fn get_playback_progress(&self) -> f64 {
        if self.loaded_frames.is_empty() {
            return 0.0;
        }
        self.playback_position as f64 / self.loaded_frames.len() as f64
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Shared State Type / نوع الحالة المشتركة
// ═══════════════════════════════════════════════════════════════════════════════

/// Thread-safe shared state type
/// نوع الحالة المشتركة الآمنة للخيوط
pub type SharedState = Arc<Mutex<AppState>>;

/// Create a new shared state instance
/// إنشاء مثيل حالة مشتركة جديد
pub fn create_shared_state() -> SharedState {
    Arc::new(Mutex::new(AppState::new()))
}
