// ═══════════════════════════════════════════════════════════════════════════════
// 📦 ui/status_panel.rs - Status Panel Components
// ═══════════════════════════════════════════════════════════════════════════════
// Contains: Receiver status, Statistics, Detectors status, Playback bar
// ═══════════════════════════════════════════════════════════════════════════════

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::state::AppState;
use super::controls;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Main Status Panel / لوحة الحالة الرئيسية
// ═══════════════════════════════════════════════════════════════════════════════

/// Render the left status panel
/// رسم لوحة الحالة اليسرى
pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    // Split into sections / التقسيم إلى أقسام
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Receiver status / حالة المستقبل
            Constraint::Length(7),  // Stats / الإحصائيات
            Constraint::Length(9),  // Detectors / الكاشفات
            Constraint::Length(5),  // Playback bar / شريط التشغيل
            Constraint::Min(8),     // Controls / التحكم
        ])
        .split(area);

    // Render each section / رسم كل قسم
    render_receiver_status(frame, chunks[0], state);
    render_stats(frame, chunks[1], state);
    render_detectors(frame, chunks[2], state);
    render_playback_bar(frame, chunks[3], state);
    controls::render(frame, chunks[4], state);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Receiver Status / حالة المستقبل
// ═══════════════════════════════════════════════════════════════════════════════

/// Render receiver status box
/// رسم مربع حالة المستقبل
fn render_receiver_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let (status_text, status_color) = if state.receiver_active {
        ("● ACTIVE", Color::Green)
    } else {
        ("○ STOPPED", Color::Red)
    };

    let text = vec![
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(Span::raw(&state.status_message)),
    ];

    let block = Block::default()
        .title("📡 Receiver")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Statistics / الإحصائيات
// ═══════════════════════════════════════════════════════════════════════════════

/// Render statistics box
/// رسم مربع الإحصائيات
fn render_stats(frame: &mut Frame, area: Rect, state: &AppState) {
    // Get Wi-Fi standard based on subcarrier count
    let wifi_info = crate::detectors::get_subcarrier_info(state.max_sc);

    let text = vec![
        Line::from(vec![
            Span::raw("Frames: "),
            Span::styled(
                format!("{}", state.frame_count()),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::raw("SC: "),
            Span::styled(
                format!("{}", state.max_sc),
                Style::default().fg(Color::Magenta),
            ),
            Span::raw(" "),
            Span::styled(
                wifi_info.wifi_standard,
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("Analysis: "),
            Span::styled(
                format!("[{}-{}]", wifi_info.analysis_range.0, wifi_info.analysis_range.1),
                Style::default().fg(Color::Green),
            ),
            Span::raw(format!(" ({})", wifi_info.analysis_range.1 - wifi_info.analysis_range.0)),
        ]),
        Line::from(vec![
            Span::raw("Port: "),
            Span::styled(&state.port_name, Style::default().fg(Color::Cyan)),
        ]),
    ];

    let block = Block::default()
        .title("📊 Statistics")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Detectors Status / حالة الكاشفات
// ═══════════════════════════════════════════════════════════════════════════════

/// Render detectors status box
/// رسم مربع حالة الكاشفات
fn render_detectors(frame: &mut Frame, area: Rect, state: &AppState) {
    let motion_status = if state.detections.motion_detected {
        ("🔴 DETECTED", Color::Red)
    } else {
        ("🟢 None", Color::Green)
    };

    let human_status = if state.detections.human_present {
        ("🔴 PRESENT", Color::Red)
    } else {
        ("🟢 Not Present", Color::Green)
    };

    let door_status = if state.detections.door_open {
        ("🔴 OPEN", Color::Red)
    } else {
        ("🟢 Closed", Color::Green)
    };

    let text = vec![
        Line::from(vec![
            Span::raw("Motion: "),
            Span::styled(motion_status.0, Style::default().fg(motion_status.1)),
            Span::styled(format!(" ({:.1})", state.detections.motion_value), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("Human: "),
            Span::styled(human_status.0, Style::default().fg(human_status.1)),
            Span::styled(format!(" ({:.1})", state.detections.presence_value), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("Door: "),
            Span::styled(door_status.0, Style::default().fg(door_status.1)),
            Span::styled(format!(" ({:.1})", state.detections.door_value), Style::default().fg(Color::Yellow)),
        ]),
    ];

    let block = Block::default()
        .title("🔍 Detectors")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Playback Bar / شريط التشغيل
// ═══════════════════════════════════════════════════════════════════════════════

/// Render playback progress bar
/// رسم شريط تقدم التشغيل
fn render_playback_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    if state.playback_mode {
        let progress = state.get_playback_progress();
        let current_sec = state.get_current_playback_second();
        let total_sec = state.playback_duration_secs;
        
        let play_status = if state.playback_playing { "▶️" } else { "⏸️" };
        
        let label = format!("{} {:.1}s / {:.1}s", play_status, current_sec, total_sec);
        
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title("🎬 Playback")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
            .ratio(progress)
            .label(label);
        
        frame.render_widget(gauge, area);
    } else {
        // Show placeholder when not in playback mode
        // عرض عنصر نائب عندما لا نكون في وضع التشغيل
        let text = vec![
            Line::from(Span::styled("No file loaded", Style::default().fg(Color::DarkGray))),
        ];
        
        let block = Block::default()
            .title("🎬 Playback")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }
}
