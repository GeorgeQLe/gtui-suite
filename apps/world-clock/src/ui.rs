use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;

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
    render_clocks(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "WORLD CLOCK",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} active", app.enabled_count()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            if app.show_24h { "24h" } else { "12h" },
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Time Zones "));

    frame.render_widget(header, area);
}

fn render_clocks(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .timezones
        .iter()
        .enumerate()
        .map(|(i, tz)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let enabled_marker = if tz.enabled { "●" } else { "○" };
            let enabled_color = if tz.enabled { Color::Green } else { Color::DarkGray };

            let time = tz.current_time();
            let date = tz.current_date();

            let line = Line::from(vec![
                Span::styled(format!("{} ", enabled_marker), Style::default().fg(enabled_color)),
                Span::styled(
                    format!("{:>5} ", tz.name),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:>8} ", tz.offset_str()),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("{} ", time),
                    Style::default()
                        .fg(if tz.enabled { Color::White } else { Color::DarkGray })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    date,
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("({})", tz.city),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Clocks ({}) ", app.timezones.len())),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
