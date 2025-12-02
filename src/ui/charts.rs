// ═══════════════════════════════════════════════════════════════════════════════
// 📦 ui/charts.rs - Chart Components
// ═══════════════════════════════════════════════════════════════════════════════
// Contains: CSI magnitude chart, Detectors chart (Motion, Presence, Door)
// ═══════════════════════════════════════════════════════════════════════════════

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use crate::state::AppState;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Constants / الثوابت
// ═══════════════════════════════════════════════════════════════════════════════

/// Number of samples to display in the chart / عدد العينات للعرض في الرسم البياني
const CHART_SAMPLES: usize = 100;

/// Y-axis range for the chart / نطاق المحور الصادي للرسم البياني
const Y_AXIS_MIN: f64 = 0.0;
const Y_AXIS_MAX: f64 = 100.0;

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Chart Panel / لوحة الرسم البياني
// ═══════════════════════════════════════════════════════════════════════════════

/// Render the right chart panel
/// رسم لوحة الرسم البياني اليمنى
pub fn render_chart_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    // Split into two charts: CSI magnitude and Detectors
    // تقسيم إلى رسمين: سعة CSI والكاشفات
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // CSI Chart
            Constraint::Percentage(50), // Detectors Chart
        ])
        .split(area);

    // Render CSI magnitude chart / رسم رسم بياني سعة CSI
    render_csi_chart(frame, chunks[0], state);
    
    // Render detectors chart / رسم رسم بياني الكاشفات
    render_detectors_chart(frame, chunks[1], state);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 CSI Magnitude Chart / رسم بياني سعة CSI
// ═══════════════════════════════════════════════════════════════════════════════

/// Render the CSI magnitude chart
/// رسم رسم بياني سعة CSI
fn render_csi_chart(frame: &mut Frame, area: Rect, state: &AppState) {
    // Prepare data for the chart / تحضير البيانات للرسم البياني
    let frames = state.get_last_frames(CHART_SAMPLES);
    
    // Create data points for the chart
    // إنشاء نقاط البيانات للرسم البياني
    let data_points: Vec<(f64, f64)> = frames
        .iter()
        .enumerate()
        .map(|(i, frame)| {
            let avg_mag = if frame.mags.is_empty() {
                0.0
            } else {
                frame.mags.iter().sum::<f64>() / frame.mags.len() as f64
            };
            (i as f64, avg_mag.min(Y_AXIS_MAX))
        })
        .collect();

    let datasets = if data_points.is_empty() {
        vec![Dataset::default()
            .name("No Data")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Gray))
            .data(&[])]
    } else {
        vec![
            Dataset::default()
                .name("CSI Magnitude")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&data_points),
        ]
    };

    let x_labels = vec![
        Span::raw("0"),
        Span::raw(format!("{}", CHART_SAMPLES / 2)),
        Span::raw(format!("{}", CHART_SAMPLES)),
    ];

    let y_labels = vec![
        Span::raw(format!("{:.0}", Y_AXIS_MIN)),
        Span::raw(format!("{:.0}", Y_AXIS_MAX / 2.0)),
        Span::raw(format!("{:.0}", Y_AXIS_MAX)),
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("📈 CSI Magnitude (Last 100 Samples)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .x_axis(
            Axis::default()
                .title("Sample")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, CHART_SAMPLES as f64])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title("Magnitude")
                .style(Style::default().fg(Color::Gray))
                .bounds([Y_AXIS_MIN, Y_AXIS_MAX])
                .labels(y_labels),
        );

    frame.render_widget(chart, area);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔹 Detectors Chart / رسم بياني الكاشفات
// ═══════════════════════════════════════════════════════════════════════════════

/// Render the detectors chart with 3 lines
/// رسم رسم بياني الكاشفات مع 3 خطوط
fn render_detectors_chart(frame: &mut Frame, area: Rect, state: &AppState) {
    // Prepare motion data / تحضير بيانات الحركة
    let motion_data: Vec<(f64, f64)> = state
        .motion_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v))
        .collect();

    // Prepare presence data / تحضير بيانات الوجود
    let presence_data: Vec<(f64, f64)> = state
        .presence_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v))
        .collect();

    // Prepare door data / تحضير بيانات الباب
    let door_data: Vec<(f64, f64)> = state
        .door_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v))
        .collect();

    // Create datasets for all 3 detectors
    // إنشاء مجموعات بيانات لجميع الكاشفات الـ 3
    let datasets = vec![
        Dataset::default()
            .name("🔴 Motion")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Red))
            .data(&motion_data),
        Dataset::default()
            .name("🟢 Presence")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Green))
            .data(&presence_data),
        Dataset::default()
            .name("🔵 Door")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Blue))
            .data(&door_data),
    ];

    let x_labels = vec![
        Span::raw("0"),
        Span::raw("50"),
        Span::raw("100"),
    ];

    let y_labels = vec![
        Span::raw("0"),
        Span::raw("250"),
        Span::raw("500"),
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("🔍 Detectors (Motion | Presence | Door)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .x_axis(
            Axis::default()
                .title("Sample")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 100.0])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title("Intensity")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 500.0])  // زيادة من 100 إلى 500
                .labels(y_labels),
        );

    frame.render_widget(chart, area);
}
