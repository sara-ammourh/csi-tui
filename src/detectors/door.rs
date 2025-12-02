// ═══════════════════════════════════════════════════════════════════════════════
// 📦 detectors/door.rs - Door Detection
// ═══════════════════════════════════════════════════════════════════════════════
// كشف فتح/إغلاق الباب باستخدام بيانات CSI
// Door open/close detection using CSI data
// ═══════════════════════════════════════════════════════════════════════════════

use crate::state::{CsiFrame, DetectionResults};
use super::{get_subcarriers_with_ratio, average_magnitude};

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Constants / الثوابت
// ═══════════════════════════════════════════════════════════════════════════════

/// عتبة كشف فتح/إغلاق الباب
/// Door open/close detection threshold
pub const DOOR_THRESHOLD: f64 = 30.0;

/// إزاحة الإطارات لمقارنة كشف الباب
/// Frame offset for door detection comparison
pub const DOOR_FRAME_OFFSET: usize = 5;

/// نسبة الـ Subcarriers المستخدمة لكشف الباب (25% من المنتصف)
/// Percentage of middle subcarriers for door detection (25%)
pub const DOOR_SUBCARRIER_RATIO: f64 = 0.25;

/// مضاعف قيمة الباب للعرض
/// Door value display multiplier
pub const DOOR_DISPLAY_MULTIPLIER: f64 = 1.0;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Helper Functions / دوال مساعدة
// ═══════════════════════════════════════════════════════════════════════════════

/// استخراج الـ Subcarriers لكشف الباب (25% من المنتصف)
/// Extract subcarriers for door detection (25% from middle)
fn get_door_subcarriers(mags: &[f64]) -> &[f64] {
    get_subcarriers_with_ratio(mags, DOOR_SUBCARRIER_RATIO)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Detection Function / دالة الكشف
// ═══════════════════════════════════════════════════════════════════════════════

/// كشف فتح/إغلاق الباب من إطارات CSI
/// Detect door open/close from CSI frames
/// 
/// # Algorithm / الخوارزمية
/// ```text
/// - مقارنة الإطار الحالي مع إطار قبل 5 إطارات
/// - إذا > DOOR_THRESHOLD = باب مفتوح/مغلق
/// ```
pub fn detect_door(frames: &[CsiFrame], results: &mut DetectionResults) {
    if frames.len() <= DOOR_FRAME_OFFSET { return; }

    // استخراج الـ subcarriers للباب (25% من المنتصف)
    let last = &frames[frames.len() - 1];
    let last_door_mags = get_door_subcarriers(&last.mags);
    
    let older = &frames[frames.len() - 1 - DOOR_FRAME_OFFSET];
    let older_mags = get_door_subcarriers(&older.mags);
    
    let sc = last_door_mags.len().min(older_mags.len());
    
    let mut door_max: f64 = 0.0;
    let mut door_total: f64 = 0.0;
    
    if sc > 0 {
        for i in 0..sc {
            let diff = (last_door_mags[i] - older_mags[i]).abs();
            door_max = door_max.max(diff);
            door_total += diff;
        }
        door_total /= sc as f64;
    }
    
    // حساب درجة الباب
    let last_door_avg = average_magnitude(last_door_mags);
    let older_avg = average_magnitude(older_mags);
    let door_score = (door_max * 0.5) + (door_total * 0.3) + ((last_door_avg - older_avg).abs() * 0.2);
    
    results.door_value = door_score * DOOR_DISPLAY_MULTIPLIER;
    results.door_open = door_score > DOOR_THRESHOLD;
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
    fn test_door_detection() {
        // إنشاء 6 إطارات مع تغير كبير مفاجئ
        let mut frames = Vec::new();
        for _ in 0..5 {
            frames.push(create_test_frame(vec![10.0, 10.0, 10.0]));
        }
        // الإطار الأخير يختلف كثيراً (باب فتح)
        frames.push(create_test_frame(vec![100.0, 100.0, 100.0]));
        
        let mut results = DetectionResults::default();
        detect_door(&frames, &mut results);
        assert!(results.door_open);
    }

    #[test]
    fn test_no_door() {
        // إنشاء 6 إطارات متشابهة
        let mut frames = Vec::new();
        for i in 0..6 {
            let value = 10.0 + i as f64 * 0.1;
            frames.push(create_test_frame(vec![value, value, value]));
        }
        
        let mut results = DetectionResults::default();
        detect_door(&frames, &mut results);
        assert!(!results.door_open);
    }
}
