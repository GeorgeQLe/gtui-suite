use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Sparkline, Table},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_memory_gauge(frame, app, chunks[1]);
    render_pressure_chart(frame, app, chunks[2]);

    if app.show_processes {
        render_processes(frame, app, chunks[3]);
    } else {
        render_events(frame, app, chunks[3]);
    }

    render_status(frame, app, chunks[4]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let mem_pct = app.current_memory_percent();
    let mem_color = if mem_pct > 90.0 {
        Color::Red
    } else if mem_pct > 75.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "OOM MONITOR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Memory: {:.1}%", mem_pct),
            Style::default().fg(mem_color),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} killed", app.killed_count()),
            Style::default().fg(Color::Red),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Alert: {}%", app.alert_threshold),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" OOM Killer Monitor "));

    frame.render_widget(header, area);
}

fn render_memory_gauge(frame: &mut Frame, app: &App, area: Rect) {
    let mem_pct = app.current_memory_percent();
    let color = if mem_pct > 90.0 {
        Color::Red
    } else if mem_pct > 75.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let pressure = app.pressure_history.last();
    let label = pressure
        .map(|p| format!("{:.1}% ({} MB / {} MB)", mem_pct, p.used_mb, p.total_mb))
        .unwrap_or_else(|| format!("{:.1}%", mem_pct));

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Memory Usage "))
        .gauge_style(Style::default().fg(color))
        .ratio(mem_pct / 100.0)
        .label(label);

    frame.render_widget(gauge, area);
}

fn render_pressure_chart(frame: &mut Frame, app: &App, area: Rect) {
    let data: Vec<u64> = app.pressure_history
        .iter()
        .map(|p| ((p.used_mb as f64 / p.total_mb as f64) * 100.0) as u64)
        .collect();

    let sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" Memory Pressure History "))
        .data(&data)
        .max(100)
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(sparkline, area);
}

fn render_events(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Time", "Process", "PID", "Memory", "Score", "Status"]
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
        .events
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, event)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let status_color = if event.killed { Color::Red } else { Color::Yellow };
            let status_text = if event.killed { "KILLED" } else { "WARNING" };

            Row::new(vec![
                Cell::from(event.timestamp.clone()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(event.process.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(event.pid.to_string()),
                Cell::from(format!("{} MB", event.memory_mb)),
                Cell::from(event.oom_score.to_string()).style(Style::default().fg(Color::Yellow)),
                Cell::from(status_text).style(Style::default().fg(status_color)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(20),
        Constraint::Percentage(20),
        Constraint::Length(8),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" OOM Events ({}) - Tab to view processes ", app.events.len())),
        );

    frame.render_widget(table, area);
}

fn render_processes(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["PID", "Process", "RSS", "OOM Score", "Adj"]
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
        .processes
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, proc)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let score_color = if proc.oom_score > 800 {
                Color::Red
            } else if proc.oom_score > 500 {
                Color::Yellow
            } else {
                Color::Green
            };

            Row::new(vec![
                Cell::from(proc.pid.to_string()).style(Style::default().fg(Color::Cyan)),
                Cell::from(proc.name.clone()),
                Cell::from(format!("{} MB", proc.rss_mb)),
                Cell::from(proc.oom_score.to_string()).style(Style::default().fg(score_color)),
                Cell::from(proc.oom_score_adj.to_string()).style(Style::default().fg(Color::DarkGray)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Percentage(30),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Processes by OOM Score ({}) - Tab to view events ", app.processes.len())),
        );

    frame.render_widget(table, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
