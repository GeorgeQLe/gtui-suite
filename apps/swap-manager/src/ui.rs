use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Sparkline, Table},
};

use crate::app::{App, SwapType};

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
    render_swap_gauge(frame, app, chunks[1]);
    render_history(frame, app, chunks[2]);

    if app.show_processes {
        render_processes(frame, app, chunks[3]);
    } else {
        render_devices(frame, app, chunks[3]);
    }

    render_status(frame, app, chunks[4]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let pct = app.swap_percent();
    let color = if pct > 80.0 {
        Color::Red
    } else if pct > 50.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "SWAP MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{:.1}% used", pct),
            Style::default().fg(color),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} / {} MB", app.used_swap(), app.total_swap()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Swappiness: {}", app.swappiness),
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Swap Management "));

    frame.render_widget(header, area);
}

fn render_swap_gauge(frame: &mut Frame, app: &App, area: Rect) {
    let pct = app.swap_percent();
    let color = if pct > 80.0 {
        Color::Red
    } else if pct > 50.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Total Swap Usage "))
        .gauge_style(Style::default().fg(color))
        .ratio(pct / 100.0)
        .label(format!("{} MB / {} MB ({:.1}%)", app.used_swap(), app.total_swap(), pct));

    frame.render_widget(gauge, area);
}

fn render_history(frame: &mut Frame, app: &App, area: Rect) {
    let data: Vec<u64> = app.history.iter().map(|s| s.used_mb).collect();

    let sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" Swap Usage History "))
        .data(&data)
        .max(app.total_swap())
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(sparkline, area);
}

fn render_devices(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Device", "Type", "Size", "Used", "Priority", "Status"]
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
        .devices
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, device)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let type_color = match device.swap_type {
                SwapType::Partition => Color::Blue,
                SwapType::File => Color::Yellow,
                SwapType::Zram => Color::Magenta,
            };

            let usage_pct = device.usage_percent();
            let usage_color = if usage_pct > 80.0 {
                Color::Red
            } else if usage_pct > 50.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            let status_color = if device.enabled { Color::Green } else { Color::DarkGray };

            Row::new(vec![
                Cell::from(device.path.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(device.swap_type.name()).style(Style::default().fg(type_color)),
                Cell::from(format!("{} MB", device.size_mb)),
                Cell::from(format!("{} MB ({:.1}%)", device.used_mb, usage_pct))
                    .style(Style::default().fg(usage_color)),
                Cell::from(device.priority.to_string()),
                Cell::from(if device.enabled { "Enabled" } else { "Disabled" })
                    .style(Style::default().fg(status_color)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Percentage(25),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Percentage(20),
        Constraint::Length(10),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Swap Devices ({}) - Tab for processes ", app.devices.len())),
        );

    frame.render_widget(table, area);
}

fn render_processes(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["PID", "Process", "Swap", "RSS"]
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

            let swap_color = if proc.swap_mb > 300 {
                Color::Red
            } else if proc.swap_mb > 100 {
                Color::Yellow
            } else {
                Color::Green
            };

            Row::new(vec![
                Cell::from(proc.pid.to_string()).style(Style::default().fg(Color::Cyan)),
                Cell::from(proc.name.clone()),
                Cell::from(format!("{} MB", proc.swap_mb)).style(Style::default().fg(swap_color)),
                Cell::from(format!("{} MB", proc.rss_mb)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Percentage(40),
        Constraint::Length(15),
        Constraint::Length(15),
    ];

    let total_swap: u64 = app.processes.iter().map(|p| p.swap_mb).sum();

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Processes Using Swap ({} MB total) - Tab for devices ", total_swap)),
        );

    frame.render_widget(table, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
