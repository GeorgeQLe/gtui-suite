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

    if app.view_deps {
        render_deps(frame, app, chunks[1]);
    } else {
        render_members(frame, app, chunks[1]);
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "CARGO WORKSPACE",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} members", app.members.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} shared deps", app.shared_deps.len()),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(" | "),
        Span::styled(
            if app.view_deps { "Dependencies" } else { "Members" },
            Style::default().fg(Color::Green),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Workspace Manager "));

    frame.render_widget(header, area);
}

fn render_members(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .members
        .iter()
        .enumerate()
        .map(|(i, member)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let deps_str = if member.dependencies.is_empty() {
                String::new()
            } else {
                format!(" -> {}", member.dependencies.join(", "))
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:<15}", member.name),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!(" v{:<8}", member.version),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!(" {}", member.path),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(deps_str, Style::default().fg(Color::Magenta)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Workspace Members ({}) ", app.members.len())),
    );

    frame.render_widget(list, area);
}

fn render_deps(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .shared_deps
        .iter()
        .enumerate()
        .map(|(i, dep)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let features_str = if dep.features.is_empty() {
                String::new()
            } else {
                format!(" [{}]", dep.features.join(", "))
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:<15}", dep.name),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!(" v{:<8}", dep.version),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(features_str, Style::default().fg(Color::Green)),
                Span::styled(
                    format!(" (used by {} crates)", dep.used_by.len()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Shared Dependencies ({}) ", app.shared_deps.len())),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
