use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, View};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_input(frame, app, chunks[1]);
    render_results(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "DNS EXPLORER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} lookups", app.history.len()),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" DNS Lookup "));

    frame.render_widget(header, area);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.view == View::Input {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let cursor = if app.view == View::Input { "_" } else { "" };
    let display = if app.domain.is_empty() {
        "Enter domain (e.g., example.com)...".to_string()
    } else {
        format!("{}{}", app.domain, cursor)
    };

    let style = if app.domain.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(display)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Domain ")
                .border_style(border_style),
        );

    frame.render_widget(input, area);
}

fn render_results(frame: &mut Frame, app: &App, area: Rect) {
    if app.records.is_empty() {
        let placeholder = Paragraph::new("DNS records will appear here")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Records "));
        frame.render_widget(placeholder, area);
        return;
    }

    let items: Vec<ListItem> = app
        .records
        .iter()
        .enumerate()
        .map(|(i, record)| {
            let style = if i == app.selected && app.view == View::Results {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let type_color = match record.record_type.as_str() {
                "A" => Color::Green,
                "AAAA" => Color::Blue,
                "MX" => Color::Yellow,
                "NS" => Color::Magenta,
                "TXT" => Color::Cyan,
                _ => Color::White,
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:>5} ", record.record_type),
                    Style::default().fg(type_color),
                ),
                Span::styled(&record.value, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    format!("(TTL: {}s)", record.ttl),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Records ({}) ", app.records.len())),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
