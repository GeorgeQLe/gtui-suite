use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
};

use crate::app::{App, UuidVersion};

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
    render_type_selector(frame, app, chunks[1]);
    render_generated(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "UUID GENERATOR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} generated", app.generated.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            if app.uppercase { "UPPERCASE" } else { "lowercase" },
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Generator "));

    frame.render_widget(header, area);
}

fn render_type_selector(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<&str> = UuidVersion::all()
        .iter()
        .map(|v| v.name())
        .collect();

    let selected = match app.version {
        UuidVersion::V4Random => 0,
        UuidVersion::V7Timestamp => 1,
        UuidVersion::Ulid => 2,
        UuidVersion::NanoId => 3,
    };

    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title(" Type (1-4) "));

    frame.render_widget(tabs, area);
}

fn render_generated(frame: &mut Frame, app: &App, area: Rect) {
    if app.generated.is_empty() {
        let placeholder = Paragraph::new("Press 'g' or Enter to generate IDs")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Generated IDs "));
        frame.render_widget(placeholder, area);
        return;
    }

    let visible_height = area.height.saturating_sub(2) as usize;
    let start = if app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .generated
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, id)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let type_color = match id.version {
                UuidVersion::V4Random => Color::Green,
                UuidVersion::V7Timestamp => Color::Blue,
                UuidVersion::Ulid => Color::Magenta,
                UuidVersion::NanoId => Color::Cyan,
            };

            let type_label = match id.version {
                UuidVersion::V4Random => "v4",
                UuidVersion::V7Timestamp => "v7",
                UuidVersion::Ulid => "ULID",
                UuidVersion::NanoId => "nano",
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:>4} ", type_label),
                    Style::default().fg(type_color),
                ),
                Span::styled(&id.value, Style::default().fg(Color::White)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Generated IDs ({}) ", app.generated.len())),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
