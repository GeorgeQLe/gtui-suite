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
            "COMMIT BROWSER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} commits", app.commits.len()),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Git History "));

    frame.render_widget(header, area);
}

fn render_search(frame: &mut Frame, app: &App, area: Rect) {
    let search = Paragraph::new(format!("🔍 {}_", app.search_query))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(" Search "));
    frame.render_widget(search, area);
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let visible = app.visible_commits();

    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, commit)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let msg_color = if commit.message.starts_with("feat") {
                Color::Green
            } else if commit.message.starts_with("fix") {
                Color::Red
            } else if commit.message.starts_with("docs") {
                Color::Blue
            } else if commit.message.starts_with("test") {
                Color::Yellow
            } else {
                Color::White
            };

            let line = Line::from(vec![
                Span::styled(&commit.short_hash, Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled(
                    format!("{:<10}", truncate(&commit.author, 10)),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(&commit.message, Style::default().fg(msg_color)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Commits ({}) ", visible.len())),
    );

    frame.render_widget(list, area);
}

fn render_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(commit) = app.selected_commit() else {
        return;
    };

    let content = vec![
        Line::from(vec![
            Span::styled("Hash:    ", Style::default().fg(Color::Cyan)),
            Span::styled(&commit.hash, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Author:  ", Style::default().fg(Color::Cyan)),
            Span::raw(&commit.author),
        ]),
        Line::from(vec![
            Span::styled("Date:    ", Style::default().fg(Color::Cyan)),
            Span::raw(commit.date.format("%Y-%m-%d %H:%M:%S").to_string()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Message: ", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(format!("  {}", commit.message)),
        Line::from(""),
        Line::from(vec![
            Span::styled("Stats:   ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} files changed, ", commit.files_changed)),
            Span::styled(
                format!("+{}", commit.insertions),
                Style::default().fg(Color::Green),
            ),
            Span::raw(", "),
            Span::styled(
                format!("-{}", commit.deletions),
                Style::default().fg(Color::Red),
            ),
        ]),
    ];

    let detail = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Commit Details "))
        .wrap(Wrap { trim: false });

    frame.render_widget(detail, area);
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
        format!("{}…", &s[..max_len - 1])
    }
}
