use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
        View::List => render_list(frame, app, chunks[1]),
        View::Compare => render_compare(frame, app, chunks[1]),
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let current = app.current_branch().map(|b| b.name.as_str()).unwrap_or("none");

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "BRANCH MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("Current: ", Style::default().fg(Color::DarkGray)),
        Span::styled(current, Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::raw(if app.show_remote { "All" } else { "Local" }),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Git Branches "));

    frame.render_widget(header, area);
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let visible = app.visible_branches();

    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, branch)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let current_marker = if branch.is_current { "* " } else { "  " };
            let remote_icon = if branch.is_remote { "🌐 " } else { "  " };

            let ahead_behind = if branch.ahead > 0 || branch.behind > 0 {
                format!(" ↑{} ↓{}", branch.ahead, branch.behind)
            } else {
                String::new()
            };

            let name_color = if branch.is_current {
                Color::Green
            } else if branch.is_remote {
                Color::Red
            } else {
                Color::White
            };

            let line = Line::from(vec![
                Span::styled(current_marker, Style::default().fg(Color::Green)),
                Span::raw(remote_icon),
                Span::styled(&branch.name, Style::default().fg(name_color)),
                Span::styled(ahead_behind, Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", &branch.last_commit[..7.min(branch.last_commit.len())]),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Branches ({}) ", visible.len())),
    );

    frame.render_widget(list, area);
}

fn render_compare(frame: &mut Frame, app: &App, area: Rect) {
    let Some(idx) = app.compare_branch else {
        return;
    };

    let visible = app.visible_branches();
    let Some(branch) = visible.get(idx) else {
        return;
    };

    let current = app.current_branch();
    let current_name = current.map(|b| b.name.as_str()).unwrap_or("main");

    let content = vec![
        Line::from(vec![
            Span::styled("Comparing: ", Style::default().fg(Color::Cyan)),
            Span::styled(current_name, Style::default().fg(Color::Green)),
            Span::raw(" ↔ "),
            Span::styled(&branch.name, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Ahead:  ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} commits", branch.ahead)),
        ]),
        Line::from(vec![
            Span::styled("Behind: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} commits", branch.behind)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Last commit: ", Style::default().fg(Color::Cyan)),
            Span::raw(&branch.last_commit),
        ]),
        Line::from(vec![
            Span::styled("Date: ", Style::default().fg(Color::Cyan)),
            Span::raw(branch.last_commit_date.format("%Y-%m-%d %H:%M").to_string()),
        ]),
    ];

    let compare = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Compare "));

    frame.render_widget(compare, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
