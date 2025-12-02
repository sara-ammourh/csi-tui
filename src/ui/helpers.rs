// ═══════════════════════════════════════════════════════════════════════════════
// 📦 ui/helpers.rs - Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════
// Contains: Utility functions for UI rendering
// ═══════════════════════════════════════════════════════════════════════════════

use ratatui::layout::{Constraint, Direction, Layout, Rect};

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Helper Functions / دوال مساعدة
// ═══════════════════════════════════════════════════════════════════════════════

/// Create a centered rect with given percentage of parent area
/// إنشاء مستطيل في المنتصف بنسبة معينة من المنطقة الأصل
#[allow(dead_code)]
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
