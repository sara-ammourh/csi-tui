// ═══════════════════════════════════════════════════════════════════════════════
// 📦 serial_reader.rs - Serial Port CSI Reader
// ═══════════════════════════════════════════════════════════════════════════════
// This module handles reading CSI data from ESP32 via serial port.
// Features:
// - Runs in background thread
// - Detects CSI blocks by "mac:" delimiter
// - Uses parser to decode data
// - Pushes frames into AppState
// - Maintains last 60 seconds of data
// - Logs to CSV if logger is active
// ═══════════════════════════════════════════════════════════════════════════════

use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use chrono::Utc;

use crate::csv_logger::CsvLogger;
use crate::parser::{extract_csi_block, CsiParser};
use crate::state::{CsiFrame, SharedState};
use serialport::{available_ports, SerialPortType};

/// Automatically chooses the first available USB serial port.
pub fn auto_select_port() -> Option<String> {
    let ports = available_ports().ok()?;

    for p in ports {
        match &p.port_type {
            SerialPortType::UsbPort(_) => {
                // First USB serial device → most likely the ESP32-C3
                return Some(p.port_name.clone());
            }
            _ => {}
        }
    }

    None
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Serial Reader Configuration / إعدادات قارئ التسلسل
// ═══════════════════════════════════════════════════════════════════════════════

/// Default serial port name / اسم المنفذ التسلسلي الافتراضي
/// Used as a fallback if auto-detection fails.
pub const DEFAULT_PORT: &str = "COM3";

/// Default baud rate / معدل البود الافتراضي
pub const DEFAULT_BAUD_RATE: u32 = 115_200;

/// Read timeout in milliseconds / مهلة القراءة بالميلي ثانية
pub const READ_TIMEOUT_MS: u64 = 100;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Serial Reader Structure / هيكل قارئ التسلسل
// ═══════════════════════════════════════════════════════════════════════════════

/// Serial reader for CSI data from ESP32
/// قارئ التسلسل لبيانات CSI من ESP32
pub struct SerialReader {
    /// Port name (e.g., "COM3") / اسم المنفذ (مثل "COM3")
    port_name: String,

    /// Baud rate / معدل البود
    baud_rate: u32,

    /// Shared application state / حالة التطبيق المشتركة
    state: SharedState,

    /// Flag to stop the reader thread / علامة لإيقاف خيط القارئ
    stop_flag: Arc<AtomicBool>,

    /// Handle to the reader thread / مقبض خيط القارئ
    thread_handle: Option<JoinHandle<()>>,
}

impl SerialReader {
    /// Create a new serial reader
    /// إنشاء قارئ تسلسل جديد
    pub fn new(state: SharedState) -> Self {
        // Detect port once as initial default; will be refreshed on start()
        let detected = auto_select_port().unwrap_or(DEFAULT_PORT.to_string());

        Self {
            port_name: detected,
            baud_rate: DEFAULT_BAUD_RATE,
            state,
            stop_flag: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    /// Start the serial reader thread
    /// بدء خيط قارئ التسلسل
    pub fn start(&mut self) -> Result<(), String> {
        // Check if already running
        if self.thread_handle.is_some() {
            return Err("Serial reader already running".to_string());
        }

        // Reset stop flag
        self.stop_flag.store(false, Ordering::SeqCst);

        // 🔍 Detect serial port on startup
        let detected_port = auto_select_port().unwrap_or(self.port_name.clone());
        self.port_name = detected_port.clone();

        let port_name = detected_port;
        let baud_rate = self.baud_rate;
        let state = Arc::clone(&self.state);
        let stop_flag = Arc::clone(&self.stop_flag);

        // 🔥 UPDATE AppState.port_name SO UI CAN DISPLAY REAL PORT
        {
            let mut guard = state.lock().map_err(|e| e.to_string())?;
            guard.port_name = port_name.clone();   // <-- IMPORTANT LINE
            guard.status_message = format!("🔄 Connecting to {}...", port_name);
        }

        // Spawn the reader thread
        let handle = thread::spawn(move || {
            run_serial_reader(&port_name, baud_rate, &state, &stop_flag);
        });

        self.thread_handle = Some(handle);
        Ok(())
    }


    /// Stop the serial reader thread
    /// إيقاف خيط قارئ التسلسل
    pub fn stop(&mut self) {
        // Set stop flag / تعيين علامة الإيقاف
        self.stop_flag.store(true, Ordering::SeqCst);

        // Wait for thread to finish / انتظار انتهاء الخيط
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        // Update state / تحديث الحالة
        if let Ok(mut state_guard) = self.state.lock() {
            state_guard.receiver_active = false;
            state_guard.status_message = "⏹️ Serial reader stopped".to_string();
        }
    }
}

impl Drop for SerialReader {
    fn drop(&mut self) {
        self.stop();
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Serial Reader Thread Function / دالة خيط قارئ التسلسل
// ═══════════════════════════════════════════════════════════════════════════════

/// Main function that runs in the serial reader thread
/// الدالة الرئيسية التي تعمل في خيط قارئ التسلسل
fn run_serial_reader(
    port_name: &str,
    baud_rate: u32,
    state: &SharedState,
    stop_flag: &Arc<AtomicBool>,
    //
) {
    // Try to open the serial port / محاولة فتح المنفذ التسلسلي
    let port_result = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(READ_TIMEOUT_MS))
        .open();

    let mut port = match port_result {
        Ok(p) => {
            // Update state to show connected / تحديث الحالة لإظهار الاتصال
            if let Ok(mut state_guard) = state.lock() {
                state_guard.receiver_active = true;
                state_guard.status_message = format!("✅ Connected to {}", port_name);
            }
            p
        }
        Err(e) => {
            // Update state to show error / تحديث الحالة لإظهار الخطأ
            if let Ok(mut state_guard) = state.lock() {
                state_guard.receiver_active = false;
                state_guard.status_message =
                    format!("❌ Failed to open {}: {}", port_name, e);
            }
            return;
        }
    };

    // Create parser and CSV logger / إنشاء المحلل ومسجل CSV
    let parser = CsiParser::new();
    let mut csv_logger = CsvLogger::new_with_timestamp().ok();

    // Buffer for incoming data / مخزن مؤقت للبيانات الواردة
    let mut text_buffer = String::new();
    let mut read_buffer = [0u8; 1024];

    // Main reading loop / حلقة القراءة الرئيسية
    while !stop_flag.load(Ordering::SeqCst) {
        // Read from serial port / القراءة من المنفذ التسلسلي
        match port.read(&mut read_buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                // Convert to string and append / التحويل إلى نص والإضافة
                let text = String::from_utf8_lossy(&read_buffer[..bytes_read]);
                text_buffer.push_str(&text);

                // Process complete CSI blocks / معالجة كتل CSI المكتملة
                process_buffer(&mut text_buffer, &parser, state, &mut csv_logger);
            }
            Ok(_) => {
                // No data, continue / لا توجد بيانات، متابعة
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout is normal, continue / المهلة طبيعية، متابعة
            }
            Err(e) => {
                // Error reading, update state / خطأ في القراءة، تحديث الحالة
                if let Ok(mut state_guard) = state.lock() {
                    state_guard.status_message = format!("⚠️ Read error: {}", e);
                }
                break;
            }
        }
    }

    // Flush CSV logger before exiting / تفريغ مسجل CSV قبل الخروج
    if let Some(ref mut logger) = csv_logger {
        let _ = logger.flush();
    }

    // Update state to show stopped / تحديث الحالة لإظهار التوقف
    if let Ok(mut state_guard) = state.lock() {
        state_guard.receiver_active = false;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Buffer Processing / معالجة المخزن المؤقت
// ═══════════════════════════════════════════════════════════════════════════════

/// Process the text buffer to extract and parse CSI blocks
/// معالجة المخزن المؤقت لاستخراج وتحليل كتل CSI
fn process_buffer(
    buffer: &mut String,
    parser: &CsiParser,
    state: &SharedState,
    csv_logger: &mut Option<CsvLogger>,
) {
    // Look for complete CSI blocks delimited by "mac:"
    // البحث عن كتل CSI الكاملة المحددة بـ "mac:"
    while let Some(start) = buffer.find("mac:") {
        // Find the next "mac:" to delimit the block
        // البحث عن "mac:" التالية لتحديد الكتلة
        if let Some(end_rel) = buffer[start + 4..].find("mac:") {
            let end = start + 4 + end_rel;

            // Extract the complete block / استخراج الكتلة الكاملة
            let block = buffer[start..end].to_string();

            // Remove processed block from buffer / إزالة الكتلة المعالجة من المخزن
            buffer.replace_range(start..end, "");

            // Parse the block / تحليل الكتلة
            if let Some(csi_data) = extract_csi_block(&block) {
                if let Some(result) = parser.parse(csi_data) {
                    // Create frame with current timestamp
                    // إنشاء إطار بالطابع الزمني الحالي
                    let timestamp = Utc::now().timestamp_millis();
                    let frame = CsiFrame::new(
                        timestamp,
                        result.mags,
                        result.pairs,
                        result.format,
                    );

                    // Log to CSV if logger exists / تسجيل في CSV إذا وجد المسجل
                    if let Some(ref mut logger) = csv_logger {
                        let _ = logger.log_frame(&frame);
                    }

                    // Push to state / إضافة للحالة
                    if let Ok(mut state_guard) = state.lock() {
                        let sc_count = frame.subcarrier_count();
                        state_guard.push_frame(frame);
                        state_guard.status_message = format!(
                            "📥 Receiving CSI: {} subcarriers, {} frames",
                            sc_count,
                            state_guard.frame_count()
                        );
                    }
                }
            }
        } else {
            // Incomplete block, wait for more data
            // كتلة غير مكتملة، انتظار المزيد من البيانات
            break;
        }
    }

    // Prevent buffer from growing too large / منع نمو المخزن بشكل كبير جداً
    if buffer.len() > 10_000 {
        if let Some(last_mac) = buffer.rfind("mac:") {
            buffer.replace_range(..last_mac, "");
        } else {
            buffer.clear();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Unit Tests / اختبارات الوحدة
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::create_shared_state;

    #[test]
    fn test_serial_reader_creation() {
        let state = create_shared_state();
        let _reader = SerialReader::new(state);
    }
}
