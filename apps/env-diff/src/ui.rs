use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{App, DiffStatus};

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
    render_diff(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let left_name = app.left_env.as_ref().map(|e| e.name.as_str()).unwrap_or("None");
    let right_name = app.right_env.as_ref().map(|e| e.name.as_str()).unwrap_or("None");

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "ENV DIFF",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(left_name, Style::default().fg(Color::Green)),
        Span::raw(" ↔ "),
        Span::styled(right_name, Style::default().fg(Color::Blue)),
        Span::raw(" | "),
        Span::styled(
            format!("{:?}", app.filter),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Environment Comparison "));

    frame.render_widget(header, area);
}

fn render_summary(frame: &mut Frame, app: &App, area: Rect) {
    let summary = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("Same:{} ", app.count_by_status(DiffStatus::Same)),
            Style::default().fg(Color::Green),
        ),
        Span::styled(
            format!("Different:{} ", app.count_by_status(DiffStatus::Different)),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(
            format!("Left Only:{} ", app.count_by_status(DiffStatus::OnlyLeft)),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            format!("Right Only:{}", app.count_by_status(DiffStatus::OnlyRight)),
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Summary "));

    frame.render_widget(summary, area);
}

fn render_diff(frame: &mut Frame, app: &App, area: Rect) {
    let visible = app.visible_entries();

    let left_name = app.left_env.as_ref().map(|e| e.name.as_str()).unwrap_or("Left");
    let right_name = app.right_env.as_ref().map(|e| e.name.as_str()).unwrap_or("Right");

    let header_cells = ["Status", "Variable", left_name, right_name]
        .into_iter()
        .map(|h| {
            Cell::from(h).style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        });

    let header = Row::new(header_cells).bottom_margin(1);

    let rows: Vec<Row> = visible
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let (status_icon, status_color) = match entry.status {
                DiffStatus::Same => ("=", Color::Green),
                DiffStatus::Different => ("≠", Color::Yellow),
                DiffStatus::OnlyLeft => ("←", Color::Cyan),
                DiffStatus::OnlyRight => ("→", Color::Magenta),
            };

            let left_val = if app.show_values {
                entry.left_value.as_deref().unwrap_or("-")
            } else {
                if entry.left_value.is_some() { "***" } else { "-" }
            };

            let right_val = if app.show_values {
                entry.right_value.as_deref().unwrap_or("-")
            } else {
                if entry.right_value.is_some() { "***" } else { "-" }
            };

            let cells = vec![
                Cell::from(status_icon).style(Style::default().fg(status_color)),
                Cell::from(entry.key.as_str()).style(Style::default().fg(Color::White)),
                Cell::from(truncate(left_val, 25)).style(Style::default().fg(Color::Green)),
                Cell::from(truncate(right_val, 25)).style(Style::default().fg(Color::Blue)),
            ];

            Row::new(cells).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(6),
        Constraint::Length(20),
        Constraint::Min(20),
        Constraint::Min(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Variables ({}) ",
            visible.len()
        )));

    frame.render_widget(table, area);
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
        format!("{}…", &s[..max_len - 1])
    }
}
