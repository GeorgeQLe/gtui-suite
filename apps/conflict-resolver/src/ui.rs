use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, Resolution, ViewMode};

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

    match app.view_mode {
        ViewMode::FileList => render_file_list(frame, app, chunks[1]),
        ViewMode::HunkView => render_hunk_view(frame, app, chunks[1]),
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let (resolved, total) = app.resolved_count();
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "CONFLICT RESOLVER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} files", app.files.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{}/{} resolved", resolved, total),
            Style::default().fg(if resolved == total { Color::Green } else { Color::Red }),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Merge Conflicts "));

    frame.render_widget(header, area);
}

fn render_file_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let style = if i == app.current_file {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let resolved = file.hunks.iter().filter(|h| h.resolved.is_some()).count();
            let total = file.hunks.len();

            let status_color = if resolved == total {
                Color::Green
            } else if resolved > 0 {
                Color::Yellow
            } else {
                Color::Red
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("[{}/{}] ", resolved, total),
                    Style::default().fg(status_color),
                ),
                Span::styled(&file.path, Style::default().fg(Color::White)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Conflicted Files "),
    );

    frame.render_widget(list, area);
}

fn render_hunk_view(frame: &mut Frame, app: &App, area: Rect) {
    let Some(file) = app.current_file() else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Hunk navigation bar
    let hunk_info = format!(
        " {} - Hunk {}/{} ",
        file.path,
        file.current_hunk + 1,
        file.hunks.len()
    );

    let resolved_text = if let Some(hunk) = file.hunks.get(file.current_hunk) {
        match hunk.resolved {
            Some(Resolution::Ours) => " [RESOLVED: Ours]",
            Some(Resolution::Theirs) => " [RESOLVED: Theirs]",
            Some(Resolution::Both) => " [RESOLVED: Both]",
            Some(Resolution::Custom) => " [RESOLVED: Custom]",
            None => " [UNRESOLVED]",
        }
    } else {
        ""
    };

    let nav = Paragraph::new(Line::from(vec![
        Span::styled(&hunk_info, Style::default().fg(Color::Cyan)),
        Span::styled(
            resolved_text,
            Style::default().fg(if resolved_text.contains("UNRESOLVED") {
                Color::Red
            } else {
                Color::Green
            }),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(nav, chunks[0]);

    // Ours panel
    if let Some(hunk) = file.hunks.get(file.current_hunk) {
        let ours_text: Vec<Line> = hunk
            .ours
            .iter()
            .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::Green))))
            .collect();

        let ours = Paragraph::new(ours_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Ours (o/1) ")
                .border_style(Style::default().fg(Color::Green)),
        );

        frame.render_widget(ours, chunks[1]);

        // Theirs panel
        let theirs_text: Vec<Line> = hunk
            .theirs
            .iter()
            .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::Blue))))
            .collect();

        let theirs = Paragraph::new(theirs_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Theirs (t/2) ")
                .border_style(Style::default().fg(Color::Blue)),
        );

        frame.render_widget(theirs, chunks[2]);
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
