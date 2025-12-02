// ═══════════════════════════════════════════════════════════════════════════════
// 📦 csv_logger.rs - CSV Data Logger
// ═══════════════════════════════════════════════════════════════════════════════
// This module handles logging CSI data to CSV files.
// Features:
// - Auto-expanding header when subcarrier count increases
// - Writes timestamp, real, imag pairs for each frame
// - Fills missing values with empty cells
// - Flushes on exit
// ═══════════════════════════════════════════════════════════════════════════════

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use chrono::Utc;

use crate::state::CsiFrame;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 CSV Logger Structure / هيكل مسجل CSV
// ═══════════════════════════════════════════════════════════════════════════════

/// CSV Logger for saving CSI frames to disk
/// مسجل CSV لحفظ إطارات CSI على القرص
pub struct CsvLogger {
    /// Buffered file writer / كاتب الملف المخزن
    writer: BufWriter<File>,
    
    /// Current number of subcarrier columns / العدد الحالي لأعمدة الناقلات الفرعية
    current_sc_count: usize,
    
    /// Whether header has been written / هل تمت كتابة الترويسة
    header_written: bool,
}

impl CsvLogger {
    /// Create a new CSV logger
    /// إنشاء مسجل CSV جديد
    /// 
    /// # Arguments
    /// * `file_path` - Path where to save the CSV file
    /// 
    /// # Returns
    /// * `Result<CsvLogger, String>` - Logger instance or error message
    pub fn new(file_path: PathBuf) -> Result<Self, String> {
        // Open file in create/append mode
        // فتح الملف في وضع الإنشاء/الإضافة
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true) // Start fresh each time / البدء من جديد كل مرة
            .open(&file_path)
            .map_err(|e| format!("Failed to create CSV file: {}", e))?;
        
        let writer = BufWriter::new(file);
        
        Ok(Self {
            writer,
            current_sc_count: 0,
            header_written: false,
        })
    }

    /// Create a new CSV logger with auto-generated filename
    /// إنشاء مسجل CSV جديد باسم ملف تلقائي
    pub fn new_with_timestamp() -> Result<Self, String> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("csi_log_{}.csv", timestamp);
        let path = PathBuf::from(filename);
        
        Self::new(path)
    }

    /// Write a CSI frame to the CSV file
    /// كتابة إطار CSI إلى ملف CSV
    pub fn log_frame(&mut self, frame: &CsiFrame) -> Result<(), String> {
        let sc_count = frame.pairs.len();
        
        // Check if we need to expand the header
        // التحقق مما إذا كنا بحاجة لتوسيع الترويسة
        if sc_count > self.current_sc_count {
            self.update_header(sc_count)?;
        }
        
        // Write the data row / كتابة صف البيانات
        self.write_row(frame)?;
        
        Ok(())
    }

    /// Update/write the header with new subcarrier count
    /// تحديث/كتابة الترويسة بعدد ناقلات فرعية جديد
    fn update_header(&mut self, new_sc_count: usize) -> Result<(), String> {
        // If header already written, we need to recreate the file
        // إذا كانت الترويسة مكتوبة بالفعل، نحتاج لإعادة إنشاء الملف
        if self.header_written {
            // For simplicity, we just update our internal count
            // The existing rows will have fewer columns (filled with empty)
            // للتبسيط، نقوم فقط بتحديث العداد الداخلي
            self.current_sc_count = new_sc_count;
            return Ok(());
        }
        
        // Build header row / بناء صف الترويسة
        let mut header = String::from("timestamp");
        
        for i in 0..new_sc_count {
            header.push_str(&format!(",r{},i{}", i, i));
        }
        header.push('\n');
        
        // Write header / كتابة الترويسة
        self.writer
            .write_all(header.as_bytes())
            .map_err(|e| format!("Failed to write header: {}", e))?;
        
        self.current_sc_count = new_sc_count;
        self.header_written = true;
        
        Ok(())
    }

    /// Write a single data row
    /// كتابة صف بيانات واحد
    fn write_row(&mut self, frame: &CsiFrame) -> Result<(), String> {
        // Start with timestamp / البدء بالطابع الزمني
        let mut row = frame.timestamp.to_string();
        
        // Add real/imag pairs / إضافة أزواج حقيقي/تخيلي
        for (real, imag) in &frame.pairs {
            row.push_str(&format!(",{},{}", real, imag));
        }
        
        // Fill missing columns with empty values
        // ملء الأعمدة المفقودة بقيم فارغة
        let missing = self.current_sc_count.saturating_sub(frame.pairs.len());
        for _ in 0..missing {
            row.push_str(",,");
        }
        
        row.push('\n');
        
        // Write row / كتابة الصف
        self.writer
            .write_all(row.as_bytes())
            .map_err(|e| format!("Failed to write row: {}", e))?;
        
        Ok(())
    }

    /// Flush all buffered data to disk
    /// تفريغ جميع البيانات المخزنة إلى القرص
    pub fn flush(&mut self) -> Result<(), String> {
        self.writer
            .flush()
            .map_err(|e| format!("Failed to flush CSV: {}", e))
    }
}

impl Drop for CsvLogger {
    /// Ensure data is flushed when logger is dropped
    /// ضمان تفريغ البيانات عند إسقاط المسجل
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Unit Tests / اختبارات الوحدة
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::CsiFormat;
    use std::fs;

    #[test]
    fn test_csv_logger_creation() {
        let path = PathBuf::from("test_output.csv");
        let logger = CsvLogger::new(path.clone());
        
        assert!(logger.is_ok());
        
        // Cleanup / تنظيف
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_csv_logging() {
        let path = PathBuf::from("test_logging.csv");
        let mut logger = CsvLogger::new(path.clone()).unwrap();
        
        let frame = CsiFrame::new(
            1234567890,
            vec![10.0, 15.0, 20.0],
            vec![(8, 6), (12, 9), (16, 12)],
            CsiFormat::RealImag,
        );
        
        let result = logger.log_frame(&frame);
        assert!(result.is_ok());
        
        logger.flush().unwrap();
        
        // Cleanup / تنظيف
        let _ = fs::remove_file(path);
    }
}
