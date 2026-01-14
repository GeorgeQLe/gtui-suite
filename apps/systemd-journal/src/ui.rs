use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, Priority, View};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);

    match app.view {
        View::Entries => render_entries(frame, app, chunks[1]),
        View::Details => render_details(frame, app, chunks[1]),
        View::Units => render_units(frame, app, chunks[1]),
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let follow_indicator = if app.follow_mode {
        Span::styled(" [FOLLOW]", Style::default().fg(Color::Green))
    } else {
        Span::raw("")
    };

    let filter_info = app
        .filter_unit
        .as_ref()
        .map(|u| format!(" | Unit: {}", u))
        .unwrap_or_default();

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "SYSTEMD JOURNAL",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        follow_indicator,
        Span::raw(filter_info),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" journalctl "));

    frame.render_widget(header, area);
}

fn render_entries(frame: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_entries();

    if filtered.is_empty() {
        let empty = Paragraph::new("No entries match the current filters")
            .block(Block::default().borders(Borders::ALL).title(" Log Entries "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let priority_style = match entry.priority {
                Priority::Emergency | Priority::Alert | Priority::Critical => {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                }
                Priority::Error => Style::default().fg(Color::Red),
                Priority::Warning => Style::default().fg(Color::Yellow),
                Priority::Notice => Style::default().fg(Color::Cyan),
                Priority::Info => Style::default().fg(Color::Green),
                Priority::Debug => Style::default().fg(Color::DarkGray),
            };

            let time = entry.timestamp.format("%H:%M:%S").to_string();

            let line = Line::from(vec![
                Span::styled(
                    format!("{:5} ", entry.priority.as_str()),
                    priority_style,
                ),
                Span::styled(
                    format!("{} ", time),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:20} ", truncate(&entry.unit, 20)),
                    Style::default().fg(Color::Blue),
                ),
                Span::raw(truncate(&entry.message, 60)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Log Entries ({}) ", filtered.len())),
    );

    frame.render_widget(list, area);
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    let Some(entry) = app.selected_entry() else {
        return;
    };

    let priority_style = match entry.priority {
        Priority::Emergency | Priority::Alert | Priority::Critical => {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        }
        Priority::Error => Style::default().fg(Color::Red),
        Priority::Warning => Style::default().fg(Color::Yellow),
        Priority::Notice => Style::default().fg(Color::Cyan),
        Priority::Info => Style::default().fg(Color::Green),
        Priority::Debug => Style::default().fg(Color::DarkGray),
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Timestamp: ", Style::default().fg(Color::Gray)),
            Span::raw(entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string()),
        ]),
        Line::from(vec![
            Span::styled("Unit: ", Style::default().fg(Color::Gray)),
            Span::styled(&entry.unit, Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::styled("Priority: ", Style::default().fg(Color::Gray)),
            Span::styled(entry.priority.as_str(), priority_style),
        ]),
        Line::from(vec![
            Span::styled("PID: ", Style::default().fg(Color::Gray)),
            Span::raw(
                entry
                    .pid
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
            ),
        ]),
        Line::from(vec![
            Span::styled("UID: ", Style::default().fg(Color::Gray)),
            Span::raw(
                entry
                    .uid
                    .map(|u| u.to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Message:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(&entry.message[..]),
    ];

    let details = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Entry Details "))
        .wrap(Wrap { trim: false });

    frame.render_widget(details, area);
}

fn render_units(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .units
        .iter()
        .enumerate()
        .map(|(i, unit)| {
            let style = if i == app.selected_unit {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let count = app.entries.iter().filter(|e| &e.unit == unit).count();

            let line = Line::from(vec![
                Span::styled(unit, Style::default().fg(Color::Blue)),
                Span::raw(" "),
                Span::styled(
                    format!("({})", count),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Units (Enter to filter, 'a' for all) "),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
