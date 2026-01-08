use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs},
};

use crate::app::{App, InputMode, View};
use crate::models::Severity;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    render_tabs(frame, app, chunks[0]);
    render_main(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);

    // Render overlays
    match app.input_mode {
        InputMode::Search => render_search(frame, app),
        InputMode::AlertDetail => render_alert_detail(frame, app),
        InputMode::RuleEdit => render_rule_edit(frame, app),
        InputMode::Normal => {}
    }
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<&str> = View::all().iter().map(|v| v.name()).collect();
    let selected = View::all().iter().position(|v| *v == app.view).unwrap_or(0);

    let alert_count = app.active_alerts_count();
    let title = if alert_count > 0 {
        format!(" Log Anomaly Detector ({} alerts) ", alert_count)
    } else {
        " Log Anomaly Detector ".to_string()
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(selected);

    frame.render_widget(tabs, area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Dashboard => render_dashboard(frame, app, area),
        View::Alerts => render_alerts(frame, app, area),
        View::Rules => render_rules(frame, app, area),
        View::Training => render_training(frame, app, area),
    }
}

fn render_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(5)])
        .split(area);

    // Stats panel
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[0]);

    let active = app.active_alerts_count();
    let critical = app.critical_alerts_count();

    let active_style = if critical > 0 {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if active > 0 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    };

    let alerts_widget = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(active.to_string(), active_style)),
        Line::from("Active Alerts"),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    let critical_widget = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            critical.to_string(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from("Critical"),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    let rules_widget = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            app.rules.iter().filter(|r| r.enabled).count().to_string(),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from("Active Rules"),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    let lines_widget = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            format_number(app.lines_scanned),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from("Lines Scanned"),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    frame.render_widget(alerts_widget, stats_chunks[0]);
    frame.render_widget(critical_widget, stats_chunks[1]);
    frame.render_widget(rules_widget, stats_chunks[2]);
    frame.render_widget(lines_widget, stats_chunks[3]);

    // Recent alerts
    let recent: Vec<ListItem> = app
        .alerts
        .iter()
        .filter(|a| !a.acknowledged)
        .take(10)
        .map(|a| {
            let severity_style = severity_style(a.severity);
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", a.severity.as_str()), severity_style),
                Span::raw(&a.rule_name),
                Span::styled(format!(" ({}x)", a.count), Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let recent_widget = List::new(recent)
        .block(Block::default().borders(Borders::ALL).title(" Recent Alerts "));
    frame.render_widget(recent_widget, chunks[1]);
}

fn render_alerts(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Severity", "Rule", "Count", "Last Seen", "Acked"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .alerts
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(Span::styled(a.severity.as_str(), severity_style(a.severity))),
                Cell::from(a.rule_name.clone()),
                Cell::from(a.count.to_string()),
                Cell::from(a.last_seen.format("%H:%M:%S").to_string()),
                Cell::from(if a.acknowledged { "yes" } else { "no" }),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(35),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Alerts "));

    frame.render_widget(table, area);
}

fn render_rules(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Enabled", "Name", "Severity", "Description"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .rules
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(if r.enabled { "[x]" } else { "[ ]" }),
                Cell::from(r.name.clone()),
                Cell::from(Span::styled(r.severity.as_str(), severity_style(r.severity))),
                Cell::from(r.description.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(50),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Detection Rules "));

    frame.render_widget(table, area);
}

fn render_training(frame: &mut Frame, app: &App, area: Rect) {
    let baseline = &app.baseline;

    let lines = vec![
        Line::from(vec![
            Span::styled("Baseline Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(if baseline.training_complete {
                "Complete"
            } else {
                "Training..."
            }),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Total Lines: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format_number(baseline.total_lines)),
        ]),
        Line::from(vec![
            Span::styled("Error Count: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format_number(baseline.error_count)),
        ]),
        Line::from(vec![
            Span::styled("Warning Count: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format_number(baseline.warning_count)),
        ]),
        Line::from(vec![
            Span::styled("Error Rate: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:.2}%", baseline.error_rate())),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "The baseline is used to detect anomalies by comparing",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "current log patterns against historical data.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Baseline Training "));

    frame.render_widget(paragraph, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let help = match app.view {
        View::Dashboard => "Tab:switch  r:rules  t:training  R:rescan  q:quit",
        View::Alerts => "Tab:switch  Enter:details  a:ack  d:dismiss  /:search  q:quit",
        View::Rules => "Tab:switch  Enter:edit  Space:toggle  q:quit",
        View::Training => "Tab:switch  R:rescan  q:quit",
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let status_style = if app.critical_alerts_count() > 0 {
        Style::default().fg(Color::Red)
    } else if app.active_alerts_count() > 0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };

    let status_widget = Paragraph::new(status)
        .style(status_style)
        .block(Block::default().borders(Borders::ALL));

    let help_widget = Paragraph::new(help)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Right);

    frame.render_widget(status_widget, chunks[0]);
    frame.render_widget(help_widget, chunks[1]);
}

fn render_search(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 3, frame.area());
    frame.render_widget(Clear, area);

    let search = Paragraph::new(format!("/{}", app.search_query))
        .block(Block::default().borders(Borders::ALL).title(" Search "));

    frame.render_widget(search, area);
}

fn render_alert_detail(frame: &mut Frame, app: &App) {
    let area = centered_rect(80, 70, frame.area());
    frame.render_widget(Clear, area);

    let alert = app
        .selected_alert
        .and_then(|idx| app.alerts.get(idx));

    let content = if let Some(a) = alert {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Rule: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&a.rule_name),
            ]),
            Line::from(vec![
                Span::styled("Severity: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(a.severity.as_str(), severity_style(a.severity)),
            ]),
            Line::from(vec![
                Span::styled("Count: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(a.count.to_string()),
            ]),
            Line::from(vec![
                Span::styled("First Seen: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(a.first_seen.format("%Y-%m-%d %H:%M:%S").to_string()),
            ]),
            Line::from(vec![
                Span::styled("Last Seen: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(a.last_seen.format("%Y-%m-%d %H:%M:%S").to_string()),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Sample Log Entries:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ];

        for entry in a.log_entries.iter().take(5) {
            lines.push(Line::from(format!(
                "  {}:{} {}",
                entry.source,
                entry.line_number,
                if entry.content.len() > 60 {
                    format!("{}...", &entry.content[..60])
                } else {
                    entry.content.clone()
                }
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Press 'a' to acknowledge, Esc to close",
            Style::default().fg(Color::DarkGray),
        )));

        lines
    } else {
        vec![Line::from("Alert not found")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Alert Details "))
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_rule_edit(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 50, frame.area());
    frame.render_widget(Clear, area);

    let rule = app.rules.get(app.selected);

    let content = if let Some(r) = rule {
        vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&r.name),
            ]),
            Line::from(vec![
                Span::styled("Pattern: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&r.pattern),
            ]),
            Line::from(vec![
                Span::styled("Severity: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(r.severity.as_str(), severity_style(r.severity)),
            ]),
            Line::from(vec![
                Span::styled("Description: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&r.description),
            ]),
            Line::from(vec![
                Span::styled("Enabled: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(if r.enabled { "yes" } else { "no" }),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Press Space to toggle, Esc to close",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        vec![Line::from("Rule not found")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Rule Details "));

    frame.render_widget(paragraph, area);
}

fn severity_style(severity: Severity) -> Style {
    match severity {
        Severity::Critical => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        Severity::Error => Style::default().fg(Color::Red),
        Severity::Warning => Style::default().fg(Color::Yellow),
        Severity::Info => Style::default().fg(Color::Cyan),
    }
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

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
