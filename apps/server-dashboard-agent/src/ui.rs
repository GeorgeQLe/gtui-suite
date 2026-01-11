use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Gauge, List, ListItem, Paragraph, Row, Table},
};

use crate::app::{App, InputMode, View};
use crate::models::{AlertSeverity, ServerStatus};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(area);

    render_main(frame, app, chunks[0]);
    render_status(frame, app, chunks[1]);

    // Render overlays
    if app.input_mode == InputMode::Search {
        render_search(frame, app);
    }
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Dashboard => render_dashboard(frame, app, area),
        View::ServerDetail => render_server_detail(frame, app, area),
        View::Alerts => render_alerts(frame, app, area),
        View::AlertRules => render_alert_rules(frame, app, area),
        View::History => render_history(frame, app, area),
    }
}

fn render_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Status", "Hostname", "IP", "CPU", "Memory", "Disk", "Load", "Uptime"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let filtered = app.filtered_servers();

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, server)| {
            let style = if i == app.selected_server {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let status_color = match server.status {
                ServerStatus::Online => Color::Green,
                ServerStatus::Warning => Color::Yellow,
                ServerStatus::Critical => Color::Red,
                ServerStatus::Offline => Color::DarkGray,
                ServerStatus::Unknown => Color::Gray,
            };

            let cpu_color = if server.metrics.cpu_usage > 90.0 {
                Color::Red
            } else if server.metrics.cpu_usage > 70.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            let mem_pct = server.metrics.memory_percent();
            let mem_color = if mem_pct > 90.0 {
                Color::Red
            } else if mem_pct > 70.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            let disk_pct = server.metrics.disk_percent();
            let disk_color = if disk_pct > 85.0 {
                Color::Red
            } else if disk_pct > 70.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            Row::new(vec![
                Cell::from(format!("{} {}", server.status.icon(), server.status.as_str()))
                    .style(Style::default().fg(status_color)),
                Cell::from(server.hostname.clone()),
                Cell::from(server.ip_address.clone()),
                Cell::from(format!("{:.1}%", server.metrics.cpu_usage))
                    .style(Style::default().fg(cpu_color)),
                Cell::from(format!("{:.1}%", mem_pct))
                    .style(Style::default().fg(mem_color)),
                Cell::from(format!("{:.1}%", disk_pct))
                    .style(Style::default().fg(disk_color)),
                Cell::from(format!("{:.1}", server.metrics.load_1)),
                Cell::from(server.uptime_display()),
            ])
            .style(style)
        })
        .collect();

    let title = if app.search_query.is_empty() {
        format!(" Servers ({}) ", app.servers.len())
    } else {
        format!(" Servers ({} of {}) ", filtered.len(), app.servers.len())
    };

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(12),
            Constraint::Percentage(18),
            Constraint::Percentage(14),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(16),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}

fn render_server_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(server) = &app.current_server else {
        let empty = Paragraph::new("No server selected")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Min(3),
        ])
        .split(area);

    // Server info
    let info = vec![
        Line::from(vec![
            Span::styled("Hostname: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&server.hostname),
        ]),
        Line::from(vec![
            Span::styled("IP: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&server.ip_address),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{} {}", server.status.icon(), server.status.as_str()),
                Style::default().fg(match server.status {
                    ServerStatus::Online => Color::Green,
                    ServerStatus::Warning => Color::Yellow,
                    ServerStatus::Critical => Color::Red,
                    _ => Color::Gray,
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Uptime: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(server.uptime_display()),
        ]),
        Line::from(vec![
            Span::styled("Last seen: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(server.last_seen_display()),
        ]),
    ];

    let info_widget = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title(" Server Info "));
    frame.render_widget(info_widget, chunks[0]);

    // CPU gauge
    let cpu_color = if server.metrics.cpu_usage > 90.0 {
        Color::Red
    } else if server.metrics.cpu_usage > 70.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let cpu_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" CPU "))
        .gauge_style(Style::default().fg(cpu_color))
        .percent(server.metrics.cpu_usage as u16)
        .label(format!("{:.1}%", server.metrics.cpu_usage));
    frame.render_widget(cpu_gauge, chunks[1]);

    // Memory gauge
    let mem_pct = server.metrics.memory_percent();
    let mem_color = if mem_pct > 90.0 {
        Color::Red
    } else if mem_pct > 70.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let mem_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Memory "))
        .gauge_style(Style::default().fg(mem_color))
        .percent(mem_pct as u16)
        .label(format!("{:.1}% ({})", mem_pct, server.metrics.memory_display()));
    frame.render_widget(mem_gauge, chunks[2]);

    // Disk gauge
    let disk_pct = server.metrics.disk_percent();
    let disk_color = if disk_pct > 85.0 {
        Color::Red
    } else if disk_pct > 70.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let disk_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Disk "))
        .gauge_style(Style::default().fg(disk_color))
        .percent(disk_pct as u16)
        .label(format!("{:.1}% ({})", disk_pct, server.metrics.disk_display()));
    frame.render_widget(disk_gauge, chunks[3]);

    // Load and process info
    let extra = vec![
        Line::from(vec![
            Span::styled("Load: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(
                "{:.2} / {:.2} / {:.2}",
                server.metrics.load_1, server.metrics.load_5, server.metrics.load_15
            )),
        ]),
        Line::from(vec![
            Span::styled("Processes: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(server.metrics.process_count.to_string()),
        ]),
    ];

    let extra_widget = Paragraph::new(extra)
        .block(Block::default().borders(Borders::ALL).title(" System "));
    frame.render_widget(extra_widget, chunks[4]);
}

fn render_alerts(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Severity", "Server", "Metric", "Value", "Threshold", "Duration"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .alerts
        .iter()
        .enumerate()
        .map(|(i, alert)| {
            let style = if i == app.selected_alert {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let severity_color = match alert.severity {
                AlertSeverity::Critical => Color::Red,
                AlertSeverity::Warning => Color::Yellow,
                AlertSeverity::Info => Color::Cyan,
            };

            Row::new(vec![
                Cell::from(format!("{} {}", alert.severity.icon(), alert.severity.as_str()))
                    .style(Style::default().fg(severity_color)),
                Cell::from(alert.server.clone()),
                Cell::from(alert.metric.clone()),
                Cell::from(format!("{:.1}", alert.value)),
                Cell::from(format!("{:.1}", alert.threshold)),
                Cell::from(alert.duration_display()),
            ])
            .style(style)
        })
        .collect();

    let active = app.active_alerts_count();
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Alerts ({} active) ", active)),
    );

    frame.render_widget(table, area);
}

fn render_alert_rules(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Enabled", "Name", "Metric", "Condition", "Threshold", "Severity"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .alert_rules
        .iter()
        .enumerate()
        .map(|(i, rule)| {
            let style = if i == app.selected_rule {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let enabled = if rule.enabled { "✓" } else { "○" };
            let enabled_color = if rule.enabled {
                Color::Green
            } else {
                Color::Gray
            };

            Row::new(vec![
                Cell::from(enabled).style(Style::default().fg(enabled_color)),
                Cell::from(rule.name.clone()),
                Cell::from(rule.metric.clone()),
                Cell::from(rule.condition.as_str()),
                Cell::from(format!("{:.1}", rule.threshold)),
                Cell::from(rule.severity.as_str()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(10),
            Constraint::Percentage(20),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Alert Rules ({}) ", app.alert_rules.len())),
    );

    frame.render_widget(table, area);
}

fn render_history(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .history
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let style = if i == app.selected_history {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let sparkline = h.sparkline(30);
            let latest = h.values.last().map(|(_, v)| *v).unwrap_or(0.0);

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:>15}: ", h.metric),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(sparkline),
                Span::styled(format!(" {:.1}", latest), Style::default().fg(Color::Cyan)),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Metric History "));

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let connected = if app.connected { "●" } else { "○" };
    let connected_color = if app.connected {
        Color::Green
    } else {
        Color::Red
    };

    let line = Line::from(vec![
        Span::styled(format!("{} ", connected), Style::default().fg(connected_color)),
        Span::raw(status),
    ]);

    let paragraph = Paragraph::new(line).style(Style::default().bg(Color::DarkGray));
    frame.render_widget(paragraph, area);
}

fn render_search(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 3, frame.area());
    frame.render_widget(Clear, area);

    let search = Paragraph::new(format!("/{}", app.search_query))
        .block(Block::default().borders(Borders::ALL).title(" Search "));

    frame.render_widget(search, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
