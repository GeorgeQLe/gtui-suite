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
    let locked = app.worktrees.iter().filter(|w| w.is_locked).count();
    let prunable = app.worktrees.iter().filter(|w| w.is_prunable).count();

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "WORKTREE MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} worktrees", app.worktrees.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} locked", locked),
            Style::default().fg(if locked > 0 { Color::Magenta } else { Color::DarkGray }),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} prunable", prunable),
            Style::default().fg(if prunable > 0 { Color::Red } else { Color::DarkGray }),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Git Worktrees "));

    frame.render_widget(header, area);
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .worktrees
        .iter()
        .enumerate()
        .map(|(i, wt)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let status_icon = if wt.is_main {
                "★"
            } else if wt.is_locked {
                "🔒"
            } else if wt.is_prunable {
                "⚠"
            } else {
                "○"
            };

            let status_color = if wt.is_main {
                Color::Yellow
            } else if wt.is_locked {
                Color::Magenta
            } else if wt.is_prunable {
                Color::Red
            } else {
                Color::Green
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", status_icon), Style::default().fg(status_color)),
                Span::styled(
                    format!("{:<20}", wt.branch),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!(" {} ", wt.head),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(wt.path.display().to_string()),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Worktrees ({}) ", app.worktrees.len())),
    );

    frame.render_widget(list, area);
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(wt) = app.selected_worktree() {
        let details = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Path:     ", Style::default().fg(Color::Gray)),
                Span::styled(
                    wt.path.display().to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Branch:   ", Style::default().fg(Color::Gray)),
                Span::styled(&wt.branch, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  HEAD:     ", Style::default().fg(Color::Gray)),
                Span::styled(&wt.head, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Status:   ", Style::default().fg(Color::Gray)),
                if wt.is_main {
                    Span::styled("Main worktree", Style::default().fg(Color::Green))
                } else if wt.is_locked {
                    Span::styled("Locked", Style::default().fg(Color::Magenta))
                } else if wt.is_prunable {
                    Span::styled("Prunable (stale)", Style::default().fg(Color::Red))
                } else {
                    Span::styled("Active", Style::default().fg(Color::Green))
                },
            ]),
        ];

        let detail = Paragraph::new(details).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Worktree Details "),
        );

        frame.render_widget(detail, area);
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
