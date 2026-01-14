use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
};

use crate::app::{App, BumpType};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_version(frame, app, chunks[1]);
    render_bump_selector(frame, app, chunks[2]);
    render_changes(frame, app, chunks[3]);
    render_status(frame, app, chunks[4]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "VERSION BUMPER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Current: v{}", app.current_version.to_string()),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Semantic Versioning "));

    frame.render_widget(header, area);
}

fn render_version(frame: &mut Frame, app: &App, area: Rect) {
    let version_display = vec![
        Line::from(vec![
            Span::styled("  Current: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("v{}", app.current_version.to_string()),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Preview: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("v{}", app.preview_version.to_string()),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" ({})", app.bump_type.name()),
                Style::default().fg(Color::Magenta),
            ),
        ]),
    ];

    let version = Paragraph::new(version_display)
        .block(Block::default().borders(Borders::ALL).title(" Version "));

    frame.render_widget(version, area);
}

fn render_bump_selector(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Major (1)", "Minor (2)", "Patch (3)", "Pre-release (4)"];
    let selected = match app.bump_type {
        BumpType::Major => 0,
        BumpType::Minor => 1,
        BumpType::Patch => 2,
        BumpType::PreRelease => 3,
    };

    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title(" Bump Type "));

    frame.render_widget(tabs, area);
}

fn render_changes(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .changes
        .iter()
        .enumerate()
        .map(|(i, change)| {
            let style = if i == app.selected_change {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let type_color = match change.change_type.as_str() {
                "feat" => Color::Green,
                "fix" => Color::Red,
                "docs" => Color::Blue,
                "refactor" => Color::Yellow,
                "perf" => Color::Magenta,
                _ => Color::White,
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:>8}: ", change.change_type),
                    Style::default().fg(type_color),
                ),
                Span::raw(&change.description),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Changelog Entries ({}) ", app.changes.len())),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
