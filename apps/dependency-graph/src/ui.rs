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
    render_tree(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "DEPENDENCY GRAPH",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} total deps", app.total_deps()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Dev: {}", if app.show_dev { "shown" } else { "hidden" }),
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Dependencies "));

    frame.render_widget(header, area);
}

fn render_tree(frame: &mut Frame, app: &App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let start = if app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .flat_list
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, (depth, name, version, expanded))| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let indent = "  ".repeat(*depth);
            let prefix = if *expanded { "▼ " } else { "▶ " };

            let version_color = if version.starts_with("0.") {
                Color::Yellow
            } else {
                Color::Green
            };

            let line = Line::from(vec![
                Span::raw(indent),
                Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                Span::styled(name, Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(format!("v{}", version), Style::default().fg(version_color)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Tree ({}/{}) ", app.selected + 1, app.flat_list.len())),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
