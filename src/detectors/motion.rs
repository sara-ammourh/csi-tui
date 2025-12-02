// ═══════════════════════════════════════════════════════════════════════════════
// 📦 detectors/motion.rs - Motion Detection
// ═══════════════════════════════════════════════════════════════════════════════
// كشف الحركة باستخدام بيانات CSI
// Motion detection using CSI data
// ═══════════════════════════════════════════════════════════════════════════════

use crate::state::{CsiFrame, DetectionResults};
use super::{get_subcarriers_with_ratio, average_magnitude};

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Constants / الثوابت
// ═══════════════════════════════════════════════════════════════════════════════

/// عتبة كشف الحركة - إذا تجاوزت القيمة هذا الحد، يتم اكتشاف حركة
/// Motion detection threshold - values above this indicate motion
pub const MOTION_THRESHOLD: f64 = 42.0;

/// نسبة الـ Subcarriers المستخدمة لكشف الحركة (50% من المنتصف)
/// Percentage of middle subcarriers for motion detection (50%)
pub const MOTION_SUBCARRIER_RATIO: f64 = 0.50;

/// مضاعف قيمة الحركة للعرض
/// Motion value display multiplier
pub const MOTION_DISPLAY_MULTIPLIER: f64 = 5.0;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Helper Functions / دوال مساعدة
// ═══════════════════════════════════════════════════════════════════════════════

/// استخراج الـ Subcarriers لكشف الحركة (50% من المنتصف)
/// Extract subcarriers for motion detection (50% from middle)
fn get_motion_subcarriers(mags: &[f64]) -> &[f64] {
    get_subcarriers_with_ratio(mags, MOTION_SUBCARRIER_RATIO)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Detection Function / دالة الكشف
// ═══════════════════════════════════════════════════════════════════════════════

/// كشف الحركة من إطارات CSI
/// Detect motion from CSI frames
/// 
/// # Algorithm / الخوارزمية
/// ```text
/// - مقارنة آخر 3 إطارات
/// - حساب: max_diff * 0.4 + avg_diff * 0.3 + sudden_changes bonus
/// - إذا > MOTION_THRESHOLD = حركة مكتشفة
/// ```
pub fn detect_motion(frames: &[CsiFrame], results: &mut DetectionResults) {
    if frames.len() < 3 { return; }

    // الحصول على آخر 3 إطارات للمقارنة
    let last = &frames[frames.len() - 1];
    let prev = &frames[frames.len() - 2];
    let prev2 = &frames[frames.len() - 3];
    
    // استخراج الـ Subcarriers لكشف الحركة (50% من المنتصف)
    let last_mags = get_motion_subcarriers(&last.mags);
    let prev_mags = get_motion_subcarriers(&prev.mags);
    let prev2_mags = get_motion_subcarriers(&prev2.mags);
    
    // الحد الأدنى لعدد الموجات الحاملة المشتركة
    let sc_count = last_mags.len().min(prev_mags.len()).min(prev2_mags.len());

    let mut max_diff: f64 = 0.0;
    let mut total_diff: f64 = 0.0;
    let mut sudden_changes: usize = 0;
    
    if sc_count > 0 {
        for i in 0..sc_count {
            // حساب الفرق بين الإطارات المتتالية
            let diff1 = (last_mags[i] - prev_mags[i]).abs();
            let diff2 = (prev_mags[i] - prev2_mags[i]).abs();
            
            max_diff = max_diff.max(diff1).max(diff2);
            total_diff += diff1 + diff2;
            
            // تغير مفاجئ إذا تجاوز 0.1
            if diff1 > 0.1 || diff2 > 0.1 { sudden_changes += 1; }
        }
        total_diff /= sc_count as f64;
    }
    
    // حساب درجة الحركة النهائية
    let last_avg = average_magnitude(last_mags);
    let prev_avg = average_magnitude(prev_mags);
    let avg_diff = (last_avg - prev_avg).abs();
    
    // المعادلة: 40% أقصى فرق + 30% متوسط الفروقات + 30% فرق المتوسطات
    let motion_score = (max_diff * 0.4) + (total_diff * 0.3) + (avg_diff * 0.3);
    
    // مكافأة إضافية إذا كان هناك أكثر من 5 تغيرات مفاجئة
    let sc_bonus = if sudden_changes > 5 { 1.5 } else { 1.0 };
    let final_motion = motion_score * sc_bonus;
    
    results.motion_value = final_motion * MOTION_DISPLAY_MULTIPLIER;
    results.motion_detected = final_motion > MOTION_THRESHOLD;
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Unit Tests / اختبارات الوحدة
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::CsiFormat;

    fn create_test_frame(mags: Vec<f64>) -> CsiFrame {
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
        let mut results = DetectionResults::default();
        detect_motion(&frames, &mut results);
        assert!(results.motion_detected);
    }

    #[test]
    fn test_no_motion() {
        let frames = vec![
            create_test_frame(vec![10.0, 10.0, 10.0]),
            create_test_frame(vec![10.5, 10.5, 10.5]),
            create_test_frame(vec![11.0, 11.0, 11.0]),
        ];
        let mut results = DetectionResults::default();
        detect_motion(&frames, &mut results);
        assert!(!results.motion_detected);
    }
}
