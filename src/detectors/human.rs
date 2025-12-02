// ═══════════════════════════════════════════════════════════════════════════════
// 📦 detectors/human.rs - Human Presence Detection
// ═══════════════════════════════════════════════════════════════════════════════
// كشف الوجود البشري باستخدام بيانات CSI
// Human presence detection using CSI data
// ═══════════════════════════════════════════════════════════════════════════════

use crate::state::{CsiFrame, DetectionResults};
use super::{get_subcarriers_with_ratio, average_magnitude};

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Constants / الثوابت
// ═══════════════════════════════════════════════════════════════════════════════

/// الحد الأدنى لكشف الوجود البشري
/// Minimum threshold for human presence detection
pub const HUMAN_PRESENCE_MIN: f64 = 3.0;

/// الحد الأقصى لكشف الوجود البشري (لتجنب الإيجابيات الكاذبة)
/// Maximum threshold for human presence (to avoid false positives)
pub const HUMAN_PRESENCE_MAX: f64 = 50.0;

/// حجم نافذة تحليل الوجود (عدد الإطارات)
/// Presence analysis window size (number of frames)
pub const PRESENCE_WINDOW_SIZE: usize = 12;

/// نسبة الـ Subcarriers المستخدمة لكشف الوجود (35% من المنتصف)
/// Percentage of middle subcarriers for presence detection (35%)
pub const PRESENCE_SUBCARRIER_RATIO: f64 = 0.35;

/// مضاعف قيمة الوجود للعرض
/// Presence value display multiplier
pub const PRESENCE_DISPLAY_MULTIPLIER: f64 = 5.0;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Helper Functions / دوال مساعدة
// ═══════════════════════════════════════════════════════════════════════════════

/// استخراج الـ Subcarriers لكشف الوجود (35% من المنتصف)
/// Extract subcarriers for presence detection (35% from middle)
fn get_presence_subcarriers(mags: &[f64]) -> &[f64] {
    get_subcarriers_with_ratio(mags, PRESENCE_SUBCARRIER_RATIO)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Detection Function / دالة الكشف
// ═══════════════════════════════════════════════════════════════════════════════

/// كشف الوجود البشري من إطارات CSI
/// Detect human presence from CSI frames
/// 
/// # Algorithm / الخوارزمية
/// ```text
/// - تحليل آخر 12 إطار (PRESENCE_WINDOW_SIZE)
/// - حساب التباين في التغيرات الصغيرة (مثل التنفس)
/// - إذا بين HUMAN_PRESENCE_MIN و MAX = وجود بشري
/// ```
pub fn detect_presence(frames: &[CsiFrame], results: &mut DetectionResults) {
    if frames.len() < PRESENCE_WINDOW_SIZE { return; }

    // أخذ آخر 12 إطار للتحليل
    let window = &frames[frames.len() - PRESENCE_WINDOW_SIZE..];
    let mut micro_diffs: Vec<f64> = Vec::new();
    
    // حساب الفروقات الصغيرة بين كل إطارين متتاليين (35% من المنتصف)
    for i in 1..window.len() {
        let curr_mags = get_presence_subcarriers(&window[i].mags);
        let prev_w_mags = get_presence_subcarriers(&window[i - 1].mags);
        let curr = average_magnitude(curr_mags);
        let prev_w = average_magnitude(prev_w_mags);
        micro_diffs.push((curr - prev_w).abs());
    }
    
    if micro_diffs.is_empty() { return; }
    
    // حساب المتوسط والتباين للفروقات الصغيرة
    let micro_mean: f64 = micro_diffs.iter().sum::<f64>() / micro_diffs.len() as f64;
    let micro_var: f64 = micro_diffs.iter()
        .map(|&d| (d - micro_mean).powi(2))
        .sum::<f64>() / micro_diffs.len() as f64;
    
    // درجة الوجود = المتوسط + الجذر التربيعي للتباين * 2
    let presence_score = micro_mean + micro_var.sqrt() * 2.0;
    let min_act = micro_diffs.iter().cloned().fold(f64::INFINITY, f64::min);
    
    results.presence_value = presence_score * PRESENCE_DISPLAY_MULTIPLIER;
    
    // وجود بشري إذا كانت الدرجة ضمن النطاق أو هناك نشاط مستمر
    results.human_present = (presence_score > HUMAN_PRESENCE_MIN 
        && presence_score < HUMAN_PRESENCE_MAX) 
        || min_act > 0.001;
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
    fn test_presence_detection() {
        // إنشاء 12 إطار مع تغيرات صغيرة (محاكاة التنفس)
        let mut frames = Vec::new();
        for i in 0..12 {
            let value = 10.0 + (i as f64 * 0.1).sin() * 0.5;
            frames.push(create_test_frame(vec![value, value, value]));
        }
        
        let mut results = DetectionResults::default();
        detect_presence(&frames, &mut results);
        // يجب أن يكتشف تغيرات صغيرة مستمرة
        assert!(results.presence_value > 0.0);
    }

    #[test]
    fn test_no_presence() {
        // إنشاء 12 إطار متطابقة تماماً
        let mut frames = Vec::new();
        for _ in 0..12 {
            frames.push(create_test_frame(vec![10.0, 10.0, 10.0]));
        }
        
        let mut results = DetectionResults::default();
        detect_presence(&frames, &mut results);
        // لا يوجد تغيرات = لا يوجد وجود
        assert!(!results.human_present || results.presence_value < HUMAN_PRESENCE_MIN);
    }
}
