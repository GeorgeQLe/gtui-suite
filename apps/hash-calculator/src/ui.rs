use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, chunks[0]);
    render_input(frame, app, chunks[1]);
    render_hashes(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "HASH CALCULATOR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::raw("MD5, SHA-1, SHA-256, SHA-512, CRC32"),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Hasher "));

    frame.render_widget(header, area);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let display = if app.input.is_empty() {
        "Type text to calculate hashes...".to_string()
    } else {
        format!("{}_", app.input)
    };

    let style = if app.input.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(display)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input ")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(input, area);
}

fn render_hashes(frame: &mut Frame, app: &App, area: Rect) {
    if app.hashes.is_empty() {
        let placeholder = Paragraph::new("Hash values will appear here")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Hashes "));
        frame.render_widget(placeholder, area);
        return;
    }

    let items: Vec<ListItem> = app
        .hashes
        .iter()
        .enumerate()
        .map(|(i, (name, hash))| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:>8}: ", name),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(hash, Style::default().fg(Color::Green)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Hash Values "),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
