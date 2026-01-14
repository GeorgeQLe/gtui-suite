use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
};

use crate::app::{App, CommitType};

pub fn render(frame: &mut Frame, app: &App) {
    if app.preview_mode {
        render_preview(frame, app);
    } else {
        render_main(frame, app);
    }
}

fn render_main(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_commits(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let breaking = app.breaking_count();

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "CHANGELOG GENERATOR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("v{}", app.version),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} included", app.included_count()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} breaking", breaking),
            Style::default().fg(if breaking > 0 { Color::Red } else { Color::DarkGray }),
        ),
        Span::raw(" | "),
        Span::styled(
            app.output_format.name(),
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Generate Changelog "));

    frame.render_widget(header, area);
}

fn render_commits(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["", "Type", "Scope", "Message", "Author", "Date"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1);

    let visible_height = area.height.saturating_sub(3) as usize;
    let start = if app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    let rows: Vec<Row> = app
        .commits
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, commit)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let type_color = match commit.commit_type {
                CommitType::Feature => Color::Green,
                CommitType::Fix => Color::Red,
                CommitType::Docs => Color::Blue,
                CommitType::Refactor => Color::Yellow,
                CommitType::Perf => Color::Magenta,
                CommitType::Breaking => Color::LightRed,
                _ => Color::White,
            };

            let checkbox = if commit.included { "[x]" } else { "[ ]" };

            Row::new(vec![
                Cell::from(checkbox).style(Style::default().fg(if commit.included { Color::Green } else { Color::DarkGray })),
                Cell::from(commit.commit_type.name()).style(Style::default().fg(type_color)),
                Cell::from(commit.scope.clone().unwrap_or_else(|| "-".to_string())).style(Style::default().fg(Color::Cyan)),
                Cell::from(commit.message.clone()),
                Cell::from(commit.author.clone()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(commit.date.clone()).style(Style::default().fg(Color::DarkGray)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Percentage(40),
        Constraint::Length(10),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Commits ({}/{}) ", app.selected + 1, app.commits.len())),
        );

    frame.render_widget(table, area);
}

fn render_preview(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "CHANGELOG PREVIEW",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            app.output_format.name(),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(" | "),
        Span::styled(
            "Press 'p' to exit preview",
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Preview "));

    frame.render_widget(header, chunks[0]);

    let preview_lines = app.generate_preview();
    let items: Vec<ListItem> = preview_lines
        .iter()
        .map(|line| {
            let style = if line.starts_with("# ") {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if line.starts_with("## ") {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if line.starts_with("### ") {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if line.starts_with("- ") {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(Line::from(Span::styled(line.clone(), style)))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" CHANGELOG.md "));

    frame.render_widget(list, chunks[1]);

    render_status(frame, app, chunks[2]);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
