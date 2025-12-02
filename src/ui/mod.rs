// ═══════════════════════════════════════════════════════════════════════════════
// 📦 ui/mod.rs - Terminal User Interface Module
// ═══════════════════════════════════════════════════════════════════════════════
// This module implements the TUI using Ratatui.
// Features:
// - Two-column layout (Status | Chart)
// - Live magnitude graph
// - Detection status display
// - Keyboard controls display
// ═══════════════════════════════════════════════════════════════════════════════

mod charts;
mod controls;
mod helpers;
mod status_panel;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::state::SharedState;

// Re-export helpers for external use (if needed)
#[allow(unused_imports)]
pub use helpers::centered_rect;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Main Render Function / دالة الرسم الرئيسية
// ═══════════════════════════════════════════════════════════════════════════════

/// Render the entire UI
/// رسم واجهة المستخدم بالكامل
pub fn render(frame: &mut Frame, state: &SharedState) {
    // Get state data / الحصول على بيانات الحالة
    let state_guard = match state.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    // Create main layout: two columns / إنشاء التخطيط الرئيسي: عمودين
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Left panel - Status / اللوحة اليسرى - الحالة
            Constraint::Percentage(70), // Right panel - Chart / اللوحة اليمنى - الرسم البياني
        ])
        .split(frame.area());

    // Render left panel (Status) / رسم اللوحة اليسرى (الحالة)
    status_panel::render(frame, main_chunks[0], &state_guard);

    // Render right panel (Chart) / رسم اللوحة اليمنى (الرسم البياني)
    charts::render_chart_panel(frame, main_chunks[1], &state_guard);
}
