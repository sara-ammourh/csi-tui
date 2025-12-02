// ═══════════════════════════════════════════════════════════════════════════════
// 📦 ui/controls.rs - Keyboard Controls Display
// ═══════════════════════════════════════════════════════════════════════════════
// Displays available keyboard shortcuts based on current mode
// ═══════════════════════════════════════════════════════════════════════════════

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::AppState;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Controls Help Box / مربع مساعدة التحكم
// ═══════════════════════════════════════════════════════════════════════════════

/// Render controls help box
/// رسم مربع مساعدة التحكم
pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let text = if state.playback_mode {
        render_playback_controls()
    } else {
        render_normal_controls()
    };

    let block = Block::default()
        .title("⌨️ Controls")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Normal Mode Controls / أزرار الوضع العادي
// ═══════════════════════════════════════════════════════════════════════════════

/// Get controls for normal (live) mode
/// الحصول على أزرار الوضع العادي (البث المباشر)
fn render_normal_controls() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("S", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Start Serial"),
        ]),
        Line::from(vec![
            Span::styled("X", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Stop Serial"),
        ]),
        Line::from(vec![
            Span::styled("L", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Load CSV"),
        ]),
        Line::from(vec![
            Span::styled("Q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit"),
        ]),
    ]
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Playback Mode Controls / أزرار وضع التشغيل
// ═══════════════════════════════════════════════════════════════════════════════

/// Get controls for playback mode
/// الحصول على أزرار وضع التشغيل
fn render_playback_controls() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Space", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Play/Pause"),
        ]),
        Line::from(vec![
            Span::styled("←→", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" ±5s"),
        ]),
        Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" ±30s"),
        ]),
        Line::from(vec![
            Span::styled("R", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Restart"),
        ]),
        Line::from(vec![
            Span::styled("B", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(" Back to Live"),
        ]),
        Line::from(vec![
            Span::styled("Q/Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit"),
        ]),
    ]
}
