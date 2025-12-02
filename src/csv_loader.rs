// ═══════════════════════════════════════════════════════════════════════════════
// 📦 csv_loader.rs - CSV Data Loader
// ═══════════════════════════════════════════════════════════════════════════════
// This module handles loading historical CSI data from CSV files.
// Features:
// - Auto-detect number of subcarrier columns
// - Parse rows into CsiFrame structures
// - Load directly into AppState
// ═══════════════════════════════════════════════════════════════════════════════

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::state::{CsiFormat, CsiFrame, SharedState};

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 CSV Loader Structure / هيكل محمّل CSV
// ═══════════════════════════════════════════════════════════════════════════════

/// CSV Loader for reading historical CSI data
/// محمّل CSV لقراءة بيانات CSI التاريخية
pub struct CsvLoader {
    /// Number of subcarrier columns detected / عدد أعمدة الناقلات الفرعية المكتشفة
    sc_count: usize,
}

impl CsvLoader {
    /// Create a new CSV loader
    /// إنشاء محمّل CSV جديد
    pub fn new() -> Self {
        Self { sc_count: 0 }
    }

    /// Load CSI data from a CSV file
    /// تحميل بيانات CSI من ملف CSV
    /// 
    /// # Arguments
    /// * `file_path` - Path to the CSV file
    /// 
    /// # Returns
    /// * `Result<Vec<CsiFrame>, String>` - Loaded frames or error message
    pub fn load<P: AsRef<Path>>(&mut self, file_path: P) -> Result<Vec<CsiFrame>, String> {
        let file = File::open(file_path.as_ref())
            .map_err(|e| format!("Failed to open CSV file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut frames = Vec::new();
        let mut lines = reader.lines();
        
        // Parse header to detect subcarrier count
        // تحليل الترويسة لكشف عدد الناقلات الفرعية
        if let Some(header_result) = lines.next() {
            let header = header_result.map_err(|e| format!("Failed to read header: {}", e))?;
            self.parse_header(&header)?;
        } else {
            return Err("CSV file is empty".to_string());
        }
        
        // Parse data rows / تحليل صفوف البيانات
        for (line_num, line_result) in lines.enumerate() {
            let line = line_result.map_err(|e| format!("Failed to read line {}: {}", line_num + 2, e))?;
            
            if line.trim().is_empty() {
                continue;
            }
            
            match self.parse_row(&line) {
                Ok(frame) => frames.push(frame),
                Err(e) => {
                    // Log warning but continue / تسجيل تحذير ولكن المتابعة
                    eprintln!("⚠️ Warning: Skipping line {}: {}", line_num + 2, e);
                }
            }
        }
        
        Ok(frames)
    }

    /// Load CSI data directly into AppState for playback
    /// تحميل بيانات CSI مباشرة إلى AppState للتشغيل
    pub fn load_into_state<P: AsRef<Path>>(&mut self, file_path: P, state: &SharedState) -> Result<usize, String> {
        let frames = self.load(file_path)?;
        let count = frames.len();
        
        // Lock state and add frames / قفل الحالة وإضافة الإطارات
        let mut state_guard = state.lock()
            .map_err(|e| format!("Failed to lock state: {}", e))?;
        
        // Clear existing frames / مسح الإطارات الموجودة
        state_guard.clear_frames();
        
        // Store loaded frames for playback / تخزين الإطارات المحملة للتشغيل
        state_guard.loaded_frames = frames;
        
        // Calculate duration / حساب المدة
        if let (Some(first), Some(last)) = (state_guard.loaded_frames.first(), state_guard.loaded_frames.last()) {
            state_guard.playback_duration_secs = (last.timestamp - first.timestamp) as f64 / 1000.0;
        }
        
        // Start playback mode / بدء وضع التشغيل
        state_guard.start_playback();
        
        state_guard.status_message = format!(
            "✅ Loaded {} frames ({:.1}s) - Space: Play/Pause, ←→: Seek",
            count,
            state_guard.playback_duration_secs
        );
        
        Ok(count)
    }

    /// Parse the CSV header to detect column count
    /// تحليل ترويسة CSV لكشف عدد الأعمدة
    fn parse_header(&mut self, header: &str) -> Result<(), String> {
        let columns: Vec<&str> = header.split(',').collect();
        
        // Header format: timestamp,r0,i0,r1,i1,...
        // صيغة الترويسة: الطابع_الزمني,r0,i0,r1,i1,...
        // Each subcarrier has 2 columns (real, imag)
        // كل ناقل فرعي له عمودين (حقيقي، تخيلي)
        
        if columns.is_empty() {
            return Err("Empty header".to_string());
        }
        
        // First column is timestamp, rest are r/i pairs
        // العمود الأول هو الطابع الزمني، والباقي أزواج r/i
        let data_columns = columns.len() - 1;
        self.sc_count = data_columns / 2;
        
        if self.sc_count == 0 {
            return Err("No subcarrier columns found in header".to_string());
        }
        
        Ok(())
    }

    /// Parse a single data row into a CsiFrame
    /// تحليل صف بيانات واحد إلى CsiFrame
    fn parse_row(&self, row: &str) -> Result<CsiFrame, String> {
        let values: Vec<&str> = row.split(',').collect();
        
        if values.is_empty() {
            return Err("Empty row".to_string());
        }
        
        // Parse timestamp / تحليل الطابع الزمني
        let timestamp: i64 = values[0]
            .trim()
            .parse()
            .map_err(|_| "Invalid timestamp")?;
        
        // Parse real/imag pairs / تحليل أزواج حقيقي/تخيلي
        let mut pairs = Vec::new();
        let mut mags = Vec::new();
        
        let mut i = 1;
        while i + 1 < values.len() {
            let real_str = values[i].trim();
            let imag_str = values[i + 1].trim();
            
            // Skip empty values / تخطي القيم الفارغة
            if real_str.is_empty() || imag_str.is_empty() {
                i += 2;
                continue;
            }
            
            let real: i32 = real_str.parse().unwrap_or(0);
            let imag: i32 = imag_str.parse().unwrap_or(0);
            
            pairs.push((real, imag));
            
            // Calculate magnitude / حساب السعة
            let mag = ((real as f64).powi(2) + (imag as f64).powi(2)).sqrt();
            mags.push(mag);
            
            i += 2;
        }
        
        if pairs.is_empty() {
            return Err("No valid data pairs found".to_string());
        }
        
        Ok(CsiFrame::new(timestamp, mags, pairs, CsiFormat::RealImag))
    }
}

impl Default for CsvLoader {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Helper Functions / دوال مساعدة
// ═══════════════════════════════════════════════════════════════════════════════

/// Open file dialog and load CSV (uses rfd crate)
/// فتح نافذة اختيار الملف وتحميل CSV (يستخدم مكتبة rfd)
pub fn pick_and_load_csv(state: &SharedState) -> Result<usize, String> {
    // Use rfd for file dialog / استخدام rfd لنافذة الملفات
    let file = rfd::FileDialog::new()
        .add_filter("CSV Files", &["csv"])
        .add_filter("All Files", &["*"])
        .set_title("Select CSI CSV File")
        .pick_file();
    
    match file {
        Some(path) => {
            let mut loader = CsvLoader::new();
            loader.load_into_state(&path, state)
        }
        None => Err("No file selected".to_string()),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Unit Tests / اختبارات الوحدة
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let mut loader = CsvLoader::new();
        let header = "timestamp,r0,i0,r1,i1,r2,i2";
        
        loader.parse_header(header).unwrap();
        
        assert_eq!(loader.sc_count, 3);
    }

    #[test]
    fn test_parse_row() {
        let mut loader = CsvLoader::new();
        loader.sc_count = 2;
        
        let row = "1234567890,10,5,20,10";
        let frame = loader.parse_row(row).unwrap();
        
        assert_eq!(frame.timestamp, 1234567890);
        assert_eq!(frame.pairs.len(), 2);
    }
}
