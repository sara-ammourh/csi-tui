// ═══════════════════════════════════════════════════════════════════════════════
// 📦 parser.rs - CSI Data Parser
// ═══════════════════════════════════════════════════════════════════════════════
// This module parses raw CSI data from ESP32 firmware.
// Automatically detects format: Real/Imag pairs or Amplitude-only.
// Extracts numbers and computes magnitudes.
// ═══════════════════════════════════════════════════════════════════════════════

use regex::Regex;
use crate::state::CsiFormat;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Parse Result Structure / هيكل نتيجة التحليل
// ═══════════════════════════════════════════════════════════════════════════════

/// Result of parsing a CSI data block
/// نتيجة تحليل كتلة بيانات CSI
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Detected format / الصيغة المكتشفة
    pub format: CsiFormat,
    
    /// Raw (real, imag) pairs / الأزواج الخام (حقيقي، تخيلي)
    pub pairs: Vec<(i32, i32)>,
    
    /// Computed magnitudes / السعات المحسوبة
    pub mags: Vec<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 CSI Parser / محلل CSI
// ═══════════════════════════════════════════════════════════════════════════════

/// Main CSI parser with automatic format detection
/// محلل CSI الرئيسي مع كشف تلقائي للصيغة
pub struct CsiParser {
    /// Regex pattern to extract numbers from CSI data
    /// نمط التعبير النمطي لاستخراج الأرقام من بيانات CSI
    number_regex: Regex,
}

impl CsiParser {
    /// Create a new CSI parser instance
    /// إنشاء مثيل محلل CSI جديد
    pub fn new() -> Self {
        // Pattern matches integers (positive and negative)
        // النمط يطابق الأعداد الصحيحة (موجبة وسالبة)
        let number_regex = Regex::new(r"-?\d+").expect("Failed to compile regex");
        
        Self { number_regex }
    }

    /// Parse a CSI data block and return parsed result
    /// تحليل كتلة بيانات CSI وإرجاع النتيجة المحللة
    /// 
    /// # Arguments
    /// * `data` - Raw CSI data string (e.g., "[1,2,3,4,...]" or from serial)
    /// 
    /// # Returns
    /// * `Option<ParseResult>` - Parsed result or None if parsing fails
    pub fn parse(&self, data: &str) -> Option<ParseResult> {
        // Extract all numbers from the data / استخراج جميع الأرقام من البيانات
        let numbers: Vec<i32> = self.extract_numbers(data);
        
        // Need at least 2 numbers to have any meaningful data
        // نحتاج على الأقل رقمين للحصول على بيانات ذات معنى
        if numbers.is_empty() {
            return None;
        }

        // Detect format and parse accordingly / كشف الصيغة والتحليل وفقاً لها
        let (format, pairs, mags) = self.detect_and_parse(&numbers);
        
        // Return None if no valid data was parsed
        if mags.is_empty() {
            return None;
        }

        Some(ParseResult { format, pairs, mags })
    }

    /// Extract all integers from a string
    /// استخراج جميع الأعداد الصحيحة من نص
    fn extract_numbers(&self, data: &str) -> Vec<i32> {
        self.number_regex
            .find_iter(data)
            .filter_map(|m| m.as_str().parse::<i32>().ok())
            .collect()
    }

    /// Detect CSI format and parse numbers accordingly
    /// كشف صيغة CSI وتحليل الأرقام وفقاً لها
    /// 
    /// # Format Detection Logic:
    /// - If numbers come in pairs where second value is often similar magnitude
    ///   to first but with different sign pattern → Real/Imag
    /// - If numbers are all positive or mostly single-value pattern → Amplitude
    fn detect_and_parse(&self, numbers: &[i32]) -> (CsiFormat, Vec<(i32, i32)>, Vec<f64>) {
        // Heuristic: Check if this looks like Real/Imag pairs
        // استدلال: التحقق مما إذا كان هذا يشبه أزواج حقيقي/تخيلي
        let format = self.detect_format(numbers);
        
        match format {
            CsiFormat::RealImag => {
                let (pairs, mags) = self.parse_real_imag(numbers);
                (format, pairs, mags)
            }
            CsiFormat::AmplitudeOnly => {
                let (pairs, mags) = self.parse_amplitude_only(numbers);
                (format, pairs, mags)
            }
            CsiFormat::Unknown => {
                // Default to Real/Imag if even count, else Amplitude
                // افتراضياً استخدم حقيقي/تخيلي إذا كان العدد زوجي، وإلا سعة
                if numbers.len() % 2 == 0 {
                    let (pairs, mags) = self.parse_real_imag(numbers);
                    (CsiFormat::RealImag, pairs, mags)
                } else {
                    let (pairs, mags) = self.parse_amplitude_only(numbers);
                    (CsiFormat::AmplitudeOnly, pairs, mags)
                }
            }
        }
    }

    /// Detect the format of CSI data based on number patterns
    /// كشف صيغة بيانات CSI بناءً على أنماط الأرقام
    fn detect_format(&self, numbers: &[i32]) -> CsiFormat {
        if numbers.len() < 4 {
            return CsiFormat::Unknown;
        }

        // Check for Real/Imag pattern:
        // - Even number of values
        // - Mix of positive and negative numbers
        // - Pairs often have similar absolute values
        
        let has_negatives = numbers.iter().any(|&n| n < 0);
        let even_count = numbers.len() % 2 == 0;
        
        // Count how many numbers are negative
        let negative_count = numbers.iter().filter(|&&n| n < 0).count();
        let negative_ratio = negative_count as f64 / numbers.len() as f64;
        
        // Real/Imag typically has 20-50% negative values
        // حقيقي/تخيلي عادة لديه 20-50% قيم سالبة
        if has_negatives && even_count && negative_ratio > 0.15 && negative_ratio < 0.85 {
            return CsiFormat::RealImag;
        }
        
        // If all positive or mostly positive, likely amplitude
        // إذا كانت كلها موجبة أو معظمها موجب، فغالباً سعة
        if !has_negatives || negative_ratio < 0.1 {
            return CsiFormat::AmplitudeOnly;
        }
        
        CsiFormat::Unknown
    }

    /// Parse numbers as Real/Imag pairs and compute magnitudes
    /// تحليل الأرقام كأزواج حقيقي/تخيلي وحساب السعات
    fn parse_real_imag(&self, numbers: &[i32]) -> (Vec<(i32, i32)>, Vec<f64>) {
        let mut pairs = Vec::new();
        let mut mags = Vec::new();
        
        // Process pairs (real, imag)
        // معالجة الأزواج (حقيقي، تخيلي)
        for chunk in numbers.chunks(2) {
            if chunk.len() == 2 {
                let real = chunk[0];
                let imag = chunk[1];
                
                pairs.push((real, imag));
                
                // Calculate magnitude: sqrt(real² + imag²)
                // حساب السعة: الجذر التربيعي (حقيقي² + تخيلي²)
                let mag = ((real as f64).powi(2) + (imag as f64).powi(2)).sqrt();
                mags.push(mag);
            }
        }
        
        (pairs, mags)
    }

    /// Parse numbers as amplitude-only values
    /// تحليل الأرقام كقيم سعة فقط
    fn parse_amplitude_only(&self, numbers: &[i32]) -> (Vec<(i32, i32)>, Vec<f64>) {
        let mut pairs = Vec::new();
        let mut mags = Vec::new();
        
        for &num in numbers {
            // Store as (amplitude, 0) pair / تخزين كزوج (سعة، 0)
            pairs.push((num, 0));
            
            // Magnitude is the absolute value / السعة هي القيمة المطلقة
            mags.push(num.abs() as f64);
        }
        
        (pairs, mags)
    }
}

impl Default for CsiParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract CSI block from raw serial data
/// استخراج كتلة CSI من بيانات التسلسل الخام
/// 
/// Looks for data between square brackets [...]
/// يبحث عن البيانات بين الأقواس المربعة [...]
pub fn extract_csi_block(data: &str) -> Option<&str> {
    // Find the CSI data array in the format: csi_data:[...]
    // البحث عن مصفوفة بيانات CSI بالصيغة: csi_data:[...]
    if let Some(start) = data.find('[') {
        if let Some(end) = data.rfind(']') {
            if end > start {
                return Some(&data[start..=end]);
            }
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Unit Tests / اختبارات الوحدة
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_real_imag() {
        let parser = CsiParser::new();
        let data = "[10, -5, 20, -10, 15, 8]";
        
        let result = parser.parse(data).unwrap();
        
        assert_eq!(result.format, CsiFormat::RealImag);
        assert_eq!(result.pairs.len(), 3);
        assert_eq!(result.mags.len(), 3);
    }

    #[test]
    fn test_parse_amplitude_only() {
        let parser = CsiParser::new();
        let data = "[100, 150, 120, 80, 90]";
        
        let result = parser.parse(data).unwrap();
        
        assert_eq!(result.format, CsiFormat::AmplitudeOnly);
        assert_eq!(result.mags.len(), 5);
    }

    #[test]
    fn test_extract_csi_block() {
        let raw = "mac:AA:BB:CC:DD:EE:FF csi_data:[1,2,3,4,5]";
        let block = extract_csi_block(raw).unwrap();
        
        assert_eq!(block, "[1,2,3,4,5]");
    }
}
