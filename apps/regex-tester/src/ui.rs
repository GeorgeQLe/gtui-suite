use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, InputField};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_pattern(frame, app, chunks[1]);
    render_test_string(frame, app, chunks[2]);
    render_replacement(frame, app, chunks[3]);
    render_results(frame, app, chunks[4]);
    render_status(frame, app, chunks[5]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let match_info = if let Some(ref err) = app.error {
        Span::styled(
            format!("Error: {}", truncate(err, 40)),
            Style::default().fg(Color::Red),
        )
    } else {
        Span::styled(
            format!("{} matches, {} groups", app.match_count(), app.group_count()),
            Style::default().fg(Color::Green),
        )
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "REGEX TESTER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        match_info,
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Regular Expressions "));

    frame.render_widget(header, area);
}

fn render_pattern(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_field == InputField::Pattern;
    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let cursor = if is_active { "_" } else { "" };
    let content = format!("{}{}", app.pattern, cursor);

    let pattern = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Pattern ")
                .border_style(border_style),
        );

    frame.render_widget(pattern, area);
}

fn render_test_string(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_field == InputField::TestString;
    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    // Highlight matches in test string
    let mut spans = Vec::new();
    let mut last_end = 0;

    for m in &app.matches {
        if m.start > last_end {
            spans.push(Span::raw(&app.test_string[last_end..m.start]));
        }
        spans.push(Span::styled(
            &m.text,
            Style::default().bg(Color::Yellow).fg(Color::Black),
        ));
        last_end = m.end;
    }

    if last_end < app.test_string.len() {
        spans.push(Span::raw(&app.test_string[last_end..]));
    }

    if is_active {
        spans.push(Span::styled("_", Style::default().fg(Color::Yellow)));
    }

    let lines: Vec<Line> = if spans.is_empty() {
        vec![Line::from(if is_active { "_" } else { "" })]
    } else {
        vec![Line::from(spans)]
    };

    let test = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Test String ")
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(test, area);
}

fn render_replacement(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_field == InputField::Replacement;
    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let cursor = if is_active { "_" } else { "" };
    let content = format!("{}{}", app.replacement, cursor);

    let replacement = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Replacement (use $1, $2 for groups) ")
                .border_style(border_style),
        );

    frame.render_widget(replacement, area);
}

fn render_results(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_matches(frame, app, chunks[0]);
    render_replaced(frame, app, chunks[1]);
}

fn render_matches(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .matches
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let mut spans = vec![
                Span::styled(
                    format!("{}. ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("\"{}\"", &m.text),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!(" [{}-{}]", m.start, m.end),
                    Style::default().fg(Color::DarkGray),
                ),
            ];

            if !m.groups.is_empty() {
                spans.push(Span::raw(" "));
                for (j, (_, _, text)) in m.groups.iter().enumerate() {
                    spans.push(Span::styled(
                        format!("${}=\"{}\" ", j + 1, text),
                        Style::default().fg(Color::Cyan),
                    ));
                }
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Matches "));

    frame.render_widget(list, area);
}

fn render_replaced(frame: &mut Frame, app: &App, area: Rect) {
    let content = app
        .replaced_text
        .as_deref()
        .unwrap_or("Enter replacement pattern...");

    let replaced = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Replaced Text "))
        .wrap(Wrap { trim: false });

    frame.render_widget(replaced, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
