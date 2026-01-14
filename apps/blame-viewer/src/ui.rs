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
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);

    match app.view {
        View::Blame => render_blame(frame, app, chunks[1]),
        View::Commit => render_commit(frame, app, chunks[1]),
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "BLAME VIEWER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(&app.file_path, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(
            format!("{} lines", app.lines.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Git Blame "));

    frame.render_widget(header, area);
}

fn render_blame(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let hash_color = hash_to_color(&line.commit_hash);

            let spans = Line::from(vec![
                Span::styled(
                    format!("{:>4} ", line.line_number),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{} ", &line.commit_hash[..7.min(line.commit_hash.len())]),
                    Style::default().fg(hash_color),
                ),
                Span::styled(
                    format!("{:<10} ", truncate(&line.author, 10)),
                    Style::default().fg(Color::Blue),
                ),
                Span::raw(&line.content),
            ]);

            ListItem::new(spans).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Source "),
    );

    frame.render_widget(list, area);
}

fn render_commit(frame: &mut Frame, app: &App, area: Rect) {
    let Some(line) = app.selected_line() else {
        return;
    };

    let content = vec![
        Line::from(vec![
            Span::styled("Commit:  ", Style::default().fg(Color::Cyan)),
            Span::styled(&line.commit_hash, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Author:  ", Style::default().fg(Color::Cyan)),
            Span::raw(&line.author),
        ]),
        Line::from(vec![
            Span::styled("Date:    ", Style::default().fg(Color::Cyan)),
            Span::raw(line.date.format("%Y-%m-%d %H:%M:%S").to_string()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Message: ", Style::default().fg(Color::Cyan)),
            Span::raw(&line.commit_message),
        ]),
        Line::from(""),
        Line::from(Span::styled("Line:", Style::default().fg(Color::Cyan))),
        Line::from(format!("  {}", line.content)),
    ];

    let commit = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Commit Details "))
        .wrap(Wrap { trim: false });

    frame.render_widget(commit, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn hash_to_color(hash: &str) -> Color {
    let sum: u32 = hash.bytes().map(|b| b as u32).sum();
    match sum % 6 {
        0 => Color::Red,
        1 => Color::Green,
        2 => Color::Yellow,
        3 => Color::Blue,
        4 => Color::Magenta,
        _ => Color::Cyan,
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
