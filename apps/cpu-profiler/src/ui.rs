use ratatui::{
    prelude::*,
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Cell, Paragraph, Row, Sparkline, Table},
};

use crate::app::{App, ViewMode};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_cpu_chart(frame, app, chunks[1]);

    match app.view_mode {
        ViewMode::Functions => render_functions(frame, app, chunks[2]),
        ViewMode::FlameGraph => render_flame_graph(frame, app, chunks[2]),
        ViewMode::Timeline => render_timeline(frame, app, chunks[2]),
    }

    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let rec_indicator = if app.is_recording {
        Span::styled(" [REC] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("")
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "CPU PROFILER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        rec_indicator,
        Span::raw(" | "),
        Span::styled(
            format!("CPU: {:.1}%", app.current_cpu_usage()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Avg: {:.1}%", app.avg_cpu_usage()),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} samples", app.cpu_samples.len()),
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Performance Profiler "));

    frame.render_widget(header, area);
}

fn render_cpu_chart(frame: &mut Frame, app: &App, area: Rect) {
    let data: Vec<u64> = app.cpu_samples
        .iter()
        .map(|s| s.usage_percent as u64)
        .collect();

    let sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" CPU Usage "))
        .data(&data)
        .max(100)
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(sparkline, area);
}

fn render_functions(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Function", "Module", "Self %", "Self ms", "Total ms", "Calls", "Avg μs"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1);

    let visible_height = area.height.saturating_sub(3) as usize;
    let start = if app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    let rows: Vec<Row> = app
        .profiles
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, profile)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let self_pct = profile.self_percent(app.total_time_ms);
            let pct_color = if self_pct > 20.0 {
                Color::Red
            } else if self_pct > 10.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            Row::new(vec![
                Cell::from(profile.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(profile.module.clone()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(format!("{:.1}%", self_pct)).style(Style::default().fg(pct_color)),
                Cell::from(format!("{:.1}", profile.self_time_ms)),
                Cell::from(format!("{:.1}", profile.total_time_ms)),
                Cell::from(profile.call_count.to_string()),
                Cell::from(format!("{:.1}", profile.avg_time_us)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Percentage(20),
        Constraint::Percentage(25),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Functions ({}/{}) ", app.selected + 1, app.profiles.len())),
        );

    frame.render_widget(table, area);
}

fn render_flame_graph(frame: &mut Frame, app: &App, area: Rect) {
    let bars: Vec<Bar> = app.profiles
        .iter()
        .take(8)
        .map(|p| {
            let pct = p.self_percent(app.total_time_ms);
            Bar::default()
                .value(pct as u64)
                .label(Line::from(p.name.clone()))
                .style(Style::default().fg(if pct > 20.0 {
                    Color::Red
                } else if pct > 10.0 {
                    Color::Yellow
                } else {
                    Color::Green
                }))
        })
        .collect();

    let chart = BarChart::default()
        .block(Block::default().borders(Borders::ALL).title(" Flame Graph (simplified) "))
        .data(BarGroup::default().bars(&bars))
        .bar_width(8)
        .bar_gap(1)
        .max(50);

    frame.render_widget(chart, area);
}

fn render_timeline(frame: &mut Frame, app: &App, area: Rect) {
    let inner = Block::default()
        .borders(Borders::ALL)
        .title(" Timeline View ");

    let inner_area = inner.inner(area);
    frame.render_widget(inner, area);

    let lines: Vec<Line> = app.profiles
        .iter()
        .take(inner_area.height as usize)
        .map(|p| {
            let bar_width = ((p.self_percent(app.total_time_ms) / 100.0) * (inner_area.width as f64 - 30.0)) as usize;
            let bar = "█".repeat(bar_width.max(1));

            Line::from(vec![
                Span::styled(
                    format!("{:>20} ", &p.name[..p.name.len().min(20)]),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(bar, Style::default().fg(Color::Green)),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
