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

    if app.show_details {
        render_details(frame, app, chunks[1]);
    } else {
        render_list(frame, app, chunks[1]);
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "REFLOG EXPLORER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} entries", app.filtered_indices.len()),
            Style::default().fg(Color::Yellow),
        ),
        if !app.search.is_empty() {
            Span::styled(
                format!(" (filter: {})", app.search),
                Style::default().fg(Color::Magenta),
            )
        } else {
            Span::raw("")
        },
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Git Reflog "));

    frame.render_widget(header, area);
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let start = if app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .filter_map(|(i, &idx)| {
            let entry = app.entries.get(idx)?;
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let action_color = match entry.action.as_str() {
                "commit" => Color::Green,
                "commit (amend)" => Color::Yellow,
                "merge" => Color::Blue,
                "rebase" => Color::Magenta,
                "reset" => Color::Red,
                "checkout" => Color::Cyan,
                _ => Color::White,
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("HEAD@{{{}}}", entry.index),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(" "),
                Span::styled(&entry.short_id, Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(
                    format!("{:>15}", entry.action),
                    Style::default().fg(action_color),
                ),
                Span::raw(": "),
                Span::raw(&entry.message),
            ]);

            Some(ListItem::new(line).style(style))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Reflog ({}/{}) ", app.selected + 1, app.filtered_indices.len())),
    );

    frame.render_widget(list, area);
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(entry) = app.selected_entry() {
        let action_color = match entry.action.as_str() {
            "commit" => Color::Green,
            "commit (amend)" => Color::Yellow,
            "merge" => Color::Blue,
            "rebase" => Color::Magenta,
            "reset" => Color::Red,
            "checkout" => Color::Cyan,
            _ => Color::White,
        };

        let details = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Reference: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("HEAD@{{{}}}", entry.index),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Commit:    ", Style::default().fg(Color::Gray)),
                Span::styled(&entry.short_id, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Action:    ", Style::default().fg(Color::Gray)),
                Span::styled(&entry.action, Style::default().fg(action_color)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Message:   ", Style::default().fg(Color::Gray)),
                Span::raw(&entry.message),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Time:      ", Style::default().fg(Color::Gray)),
                Span::styled(&entry.timestamp, Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Commands:", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("r", Style::default().fg(Color::Yellow)),
                Span::raw(" - git reset --hard "),
                Span::styled(&entry.short_id, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("c", Style::default().fg(Color::Yellow)),
                Span::raw(" - git checkout "),
                Span::styled(&entry.short_id, Style::default().fg(Color::Cyan)),
            ]),
        ];

        let detail_widget = Paragraph::new(details).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Entry Details "),
        );

        frame.render_widget(detail_widget, area);
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
