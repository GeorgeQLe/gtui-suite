use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{App, SyncStatus};

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
    render_table(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let out_of_sync = app.out_of_sync();

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "SUBMODULE MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} submodules", app.submodules.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} need sync", out_of_sync),
            Style::default().fg(if out_of_sync > 0 { Color::Red } else { Color::Green }),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Git Submodules "));

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Name", "Path", "Branch", "Commit", "Status"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .submodules
        .iter()
        .enumerate()
        .map(|(i, sm)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let status_color = match sm.status {
                SyncStatus::UpToDate => Color::Green,
                SyncStatus::Behind => Color::Yellow,
                SyncStatus::Ahead => Color::Blue,
                SyncStatus::Modified => Color::Magenta,
                SyncStatus::Uninitialized => Color::Red,
            };

            Row::new(vec![
                Cell::from(sm.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(sm.path.clone()),
                Cell::from(sm.branch.clone()).style(Style::default().fg(Color::Magenta)),
                Cell::from(if sm.commit.is_empty() { "-" } else { &sm.commit }).style(Style::default().fg(Color::DarkGray)),
                Cell::from(sm.status.name()).style(Style::default().fg(status_color)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Percentage(20),
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Submodules ({}) ", app.submodules.len())),
        );

    frame.render_widget(table, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
