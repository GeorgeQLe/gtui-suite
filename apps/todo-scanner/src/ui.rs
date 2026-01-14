use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, TodoType, View};

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
    render_summary(frame, app, chunks[1]);

    match app.view {
        View::List => render_list(frame, app, chunks[2]),
        View::Detail => render_detail(frame, app, chunks[2]),
    }

    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let filter_info = app
        .filter
        .map(|f| format!(" | Filter: {}", f.as_str()))
        .unwrap_or_default();

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "TODO SCANNER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} items", app.items.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(&filter_info),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Code Comments "));

    frame.render_widget(header, area);
}

fn render_summary(frame: &mut Frame, app: &App, area: Rect) {
    let summary = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("TODO:{} ", app.count_by_type(TodoType::Todo)),
            Style::default().fg(Color::Green),
        ),
        Span::styled(
            format!("FIXME:{} ", app.count_by_type(TodoType::Fixme)),
            Style::default().fg(Color::Red),
        ),
        Span::styled(
            format!("HACK:{} ", app.count_by_type(TodoType::Hack)),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(
            format!("NOTE:{} ", app.count_by_type(TodoType::Note)),
            Style::default().fg(Color::Blue),
        ),
        Span::styled(
            format!("BUG:{} ", app.count_by_type(TodoType::Bug)),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled(
            format!("XXX:{}", app.count_by_type(TodoType::Xxx)),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Summary "));

    frame.render_widget(summary, area);
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let visible = app.visible_items();

    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let type_color = match item.todo_type {
                TodoType::Todo => Color::Green,
                TodoType::Fixme => Color::Red,
                TodoType::Hack => Color::Yellow,
                TodoType::Note => Color::Blue,
                TodoType::Bug => Color::Magenta,
                TodoType::Xxx => Color::DarkGray,
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:>5} ", item.todo_type.as_str()),
                    Style::default().fg(type_color),
                ),
                Span::styled(
                    format!("{}:{} ", item.file, item.line),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    truncate(&item.content, 50),
                    Style::default().fg(Color::White),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Items ({}) ", visible.len())),
    );

    frame.render_widget(list, area);
}

fn render_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(item) = app.selected_item() else {
        return;
    };

    let type_color = match item.todo_type {
        TodoType::Todo => Color::Green,
        TodoType::Fixme => Color::Red,
        TodoType::Hack => Color::Yellow,
        TodoType::Note => Color::Blue,
        TodoType::Bug => Color::Magenta,
        TodoType::Xxx => Color::DarkGray,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(5)])
        .split(area);

    // Info section
    let info = vec![
        Line::from(vec![
            Span::styled("Type:    ", Style::default().fg(Color::Cyan)),
            Span::styled(item.todo_type.as_str(), Style::default().fg(type_color)),
        ]),
        Line::from(vec![
            Span::styled("File:    ", Style::default().fg(Color::Cyan)),
            Span::raw(&item.file),
        ]),
        Line::from(vec![
            Span::styled("Line:    ", Style::default().fg(Color::Cyan)),
            Span::raw(item.line.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Content: ", Style::default().fg(Color::Cyan)),
            Span::raw(&item.content),
        ]),
    ];

    let info_widget = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title(" Details "))
        .wrap(Wrap { trim: false });

    frame.render_widget(info_widget, chunks[0]);

    // Context section
    let context = Paragraph::new(item.context.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Context "))
        .wrap(Wrap { trim: false });

    frame.render_widget(context, chunks[1]);
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
