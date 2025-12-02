// ═══════════════════════════════════════════════════════════════════════════════
// 📦 detectors/mod.rs - Activity Detection Module
// ═══════════════════════════════════════════════════════════════════════════════
// كشف النشاط باستخدام بيانات CSI (الحركة، الوجود البشري، فتح الباب)
// Activity detection using CSI data (motion, human presence, door)
// ═══════════════════════════════════════════════════════════════════════════════

mod motion;
mod human;
mod door;

use crate::state::{CsiFrame, DetectionResults};

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Structures / الهياكل
// ═══════════════════════════════════════════════════════════════════════════════

/// معلومات عن الموجات الحاملة الفرعية
/// Information about subcarriers based on WiFi standard
pub struct SubcarrierInfo {
    /// معيار الواي فاي المستخدم (Wi-Fi 4/5/6)
    /// WiFi standard being used
    pub wifi_standard: String,
    
    /// نطاق الـ Subcarriers المستخدمة للتحليل (البداية، النهاية)
    /// Range of subcarriers used for analysis (start, end)
    pub analysis_range: (usize, usize),
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Subcarrier Analysis / تحليل الموجات الحاملة الفرعية
// ═══════════════════════════════════════════════════════════════════════════════

/// تحديد معيار الواي فاي ونطاق التحليل بناءً على عدد الموجات الحاملة
/// Determine WiFi standard and analysis range based on subcarrier count
/// 
/// # كيفية اختيار الـ Subcarriers / How subcarriers are selected:
/// ```text
/// مثال: 64 subcarrier مع نسبة 25%
/// Example: 64 subcarriers with 25% ratio
/// 
/// [0..23] [24..40] [41..63]
///   ↑        ↑        ↑
/// تجاهل   تحليل    تجاهل
/// skip    analyze   skip
/// 
/// نأخذ 25% من المنتصف = 16 subcarrier
/// We take 25% from middle = 16 subcarriers
/// start = (64 - 16) / 2 = 24
/// end = 24 + 16 = 40
/// ```
pub fn get_subcarrier_info(total_sc: usize) -> SubcarrierInfo {
    // نستخدم نسبة الحركة كنسبة افتراضية للعرض في الواجهة
    get_subcarrier_info_with_ratio(total_sc, motion::MOTION_SUBCARRIER_RATIO)
}

/// تحديد معيار الواي فاي ونطاق التحليل مع نسبة محددة
/// Determine WiFi standard and analysis range with specific ratio
pub(crate) fn get_subcarrier_info_with_ratio(total_sc: usize, ratio: f64) -> SubcarrierInfo {
    let wifi_standard = match total_sc {
        0..=32 => "Unknown",
        33..=64 => "Wi-Fi 4/5 (20MHz)",
        65..=128 => "Wi-Fi 5 (40MHz)",
        129..=192 => "Wi-Fi 6 (20MHz)",
        193..=256 => "Wi-Fi 6 (40MHz)",
        _ => "Wi-Fi 6+ (80MHz+)",
    };
    
    // حساب نطاق التحليل بناءً على النسبة المحددة
    // Calculate analysis range based on specified ratio
    let analysis_count = ((total_sc as f64) * ratio).max(1.0) as usize;
    let start = (total_sc.saturating_sub(analysis_count)) / 2;
    let end = start + analysis_count;
    
    SubcarrierInfo { 
        wifi_standard: wifi_standard.to_string(),
        analysis_range: (start, end),
    }
}

/// الحصول على الـ Subcarriers بنسبة محددة من المنتصف
/// Get subcarriers with specified ratio from middle
pub(crate) fn get_subcarriers_with_ratio(mags: &[f64], ratio: f64) -> &[f64] {
    if mags.is_empty() { return mags; }
    
    let info = get_subcarrier_info_with_ratio(mags.len(), ratio);
    let (start, end) = info.analysis_range;
    
    // التأكد من عدم تجاوز الحدود
    let start = start.min(mags.len());
    let end = end.min(mags.len());
    
    &mags[start..end]
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Helper Functions / دوال مساعدة
// ═══════════════════════════════════════════════════════════════════════════════

/// حساب متوسط السعات لمصفوفة من القيم
/// Calculate average magnitude from an array of values
pub(crate) fn average_magnitude(mags: &[f64]) -> f64 {
    if mags.is_empty() { return 0.0; }
    mags.iter().sum::<f64>() / mags.len() as f64
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Main Detection Function / دالة الكشف الرئيسية
// ═══════════════════════════════════════════════════════════════════════════════

/// الكشف السريع عن النشاط (الحركة، الوجود، الباب)
/// Quick activity detection (motion, presence, door)
/// 
/// تحلل هذه الدالة آخر إطارات CSI لاكتشاف:
/// This function analyzes recent CSI frames to detect:
/// 
/// 1. **الحركة / Motion**: تغيرات مفاجئة وكبيرة في السعات
/// 2. **الوجود البشري / Human Presence**: تغيرات صغيرة ومستمرة
/// 3. **فتح/إغلاق الباب / Door Open/Close**: تغيرات كبيرة مقارنة بإطارات سابقة
pub fn quick_detect(frames: &[CsiFrame]) -> DetectionResults {
    let mut results = DetectionResults::default();
    
    // نحتاج على الأقل 3 إطارات للتحليل
    if frames.len() < 3 { return results; }

    // كشف الحركة
    motion::detect_motion(frames, &mut results);
    
    // كشف الوجود البشري
    human::detect_presence(frames, &mut results);
    
    // كشف الباب
    door::detect_door(frames, &mut results);

    results
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Unit Tests / اختبارات الوحدة
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::CsiFormat;

    /// إنشاء إطار اختبار بسعات محددة
    pub fn create_test_frame(mags: Vec<f64>) -> CsiFrame {
        let pairs: Vec<(i32, i32)> = mags.iter().map(|&m| (m as i32, 0)).collect();
        CsiFrame::new(0, mags, pairs, CsiFormat::AmplitudeOnly)
    }

    #[test]
    fn test_motion_detection() {
        let frames = vec![
            create_test_frame(vec![10.0, 10.0, 10.0]),
            create_test_frame(vec![20.0, 20.0, 20.0]),
            create_test_frame(vec![50.0, 50.0, 50.0]),
        ];
        let results = quick_detect(&frames);
        assert!(results.motion_detected);
    }

    #[test]
    fn test_no_motion() {
        let frames = vec![
            create_test_frame(vec![10.0, 10.0, 10.0]),
            create_test_frame(vec![10.5, 10.5, 10.5]),
            create_test_frame(vec![11.0, 11.0, 11.0]),
        ];
        let results = quick_detect(&frames);
        assert!(!results.motion_detected);
    }

    #[test]
    fn test_average_magnitude() {
        let mags = vec![10.0, 20.0, 30.0];
        let avg = average_magnitude(&mags);
        assert!((avg - 20.0).abs() < 0.001);
    }
}
