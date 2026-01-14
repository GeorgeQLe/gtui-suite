use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, FileStatus, View};

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
        View::Details => render_details(frame, app, chunks[1]),
        View::Diff => render_diff(frame, app, chunks[1]),
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let view_name = match app.view {
        View::List => "Stash List",
        View::Details => "Stash Details",
        View::Diff => "File Diff",
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "GIT STASH MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(view_name, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::raw(format!("{} stashes", app.stashes.len())),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Stash "));

    frame.render_widget(header, area);
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    if app.stashes.is_empty() {
        let empty = Paragraph::new("No stashes found. Use 'git stash' to create one.")
            .block(Block::default().borders(Borders::ALL).title(" Stashes "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .stashes
        .iter()
        .enumerate()
        .map(|(i, stash)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let age = format_age(stash.created_at);
            let files_count = stash.files.len();

            let line = Line::from(vec![
                Span::styled(
                    format!("stash@{{{}}} ", stash.index),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(&stash.message, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", stash.branch),
                    Style::default().fg(Color::Magenta),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{} files", files_count),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(" "),
                Span::styled(age, Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Stashes "));

    frame.render_widget(list, area);
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    let Some(stash) = app.selected_stash() else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(5)])
        .split(area);

    // Stash info
    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Message: ", Style::default().fg(Color::Gray)),
            Span::raw(&stash.message),
        ]),
        Line::from(vec![
            Span::styled("Branch: ", Style::default().fg(Color::Gray)),
            Span::styled(&stash.branch, Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::Gray)),
            Span::raw(stash.created_at.format("%Y-%m-%d %H:%M:%S").to_string()),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title(format!(
        " stash@{{{}}} ",
        stash.index
    )));

    frame.render_widget(info, chunks[0]);

    // Files list
    let items: Vec<ListItem> = stash
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let style = if i == app.selected_file {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let status_style = match file.status {
                FileStatus::Modified => Style::default().fg(Color::Yellow),
                FileStatus::Added => Style::default().fg(Color::Green),
                FileStatus::Deleted => Style::default().fg(Color::Red),
                FileStatus::Renamed => Style::default().fg(Color::Cyan),
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", file.status.icon()),
                    status_style,
                ),
                Span::styled(&file.path, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    format!("+{}", file.additions),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("-{}", file.deletions),
                    Style::default().fg(Color::Red),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Files "));

    frame.render_widget(list, chunks[1]);
}

fn render_diff(frame: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = app
        .diff_content
        .lines()
        .skip(app.scroll_offset)
        .map(|line| {
            let style = if line.starts_with('+') && !line.starts_with("+++") {
                Style::default().fg(Color::Green)
            } else if line.starts_with('-') && !line.starts_with("---") {
                Style::default().fg(Color::Red)
            } else if line.starts_with("@@") {
                Style::default().fg(Color::Cyan)
            } else if line.starts_with("diff") || line.starts_with("index") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            Line::styled(line, style)
        })
        .collect();

    let diff = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Diff "))
        .wrap(Wrap { trim: false });

    frame.render_widget(diff, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn format_age(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now - dt;

    if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours ago", duration.num_hours())
    } else {
        format!("{} mins ago", duration.num_minutes())
    }
}
