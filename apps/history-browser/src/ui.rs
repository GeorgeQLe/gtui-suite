use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, View};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            if app.view == View::Search {
                Constraint::Length(3)
            } else {
                Constraint::Length(0)
            },
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);

    if app.view == View::Search {
        render_search(frame, app, chunks[1]);
    }

    match app.view {
        View::List | View::Search => render_list(frame, app, chunks[2]),
        View::Detail => render_detail(frame, app, chunks[2]),
    }

    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "HISTORY BROWSER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} commands", app.entries.len()),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Shell History "));

    frame.render_widget(header, area);
}

fn render_search(frame: &mut Frame, app: &App, area: Rect) {
    let search = Paragraph::new(format!("🔍 {}_", app.search_query))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(" Search "));
    frame.render_widget(search, area);
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let visible = app.visible_entries();

    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:>3}x ", entry.count),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&entry.command, Style::default().fg(Color::White)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Commands ({}) ", visible.len())),
    );

    frame.render_widget(list, area);
}

fn render_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(entry) = app.selected_entry() else {
        return;
    };

    let timestamp_str = entry
        .timestamp
        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let content = vec![
        Line::from(vec![
            Span::styled("Command: ", Style::default().fg(Color::Cyan)),
            Span::raw(&entry.command),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Last Run: ", Style::default().fg(Color::Cyan)),
            Span::raw(&timestamp_str),
        ]),
        Line::from(vec![
            Span::styled("Run Count: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", entry.count)),
        ]),
    ];

    let detail = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Command Details "))
        .wrap(Wrap { trim: false });

    frame.render_widget(detail, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
