use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_file_info(frame, app, chunks[1]);
    render_attrs(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let mod_indicator = if app.modified {
        Span::styled(" [MODIFIED]", Style::default().fg(Color::Red))
    } else {
        Span::raw("")
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "ATTR EDITOR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        mod_indicator,
        Span::raw(" | "),
        Span::styled(
            format!("{} attributes", app.attrs.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} bytes", app.total_size()),
            Style::default().fg(Color::Green),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Extended Attributes Editor "));

    frame.render_widget(header, area);
}

fn render_file_info(frame: &mut Frame, app: &App, area: Rect) {
    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::Gray)),
            Span::styled(&app.file.path, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Size: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{} bytes", app.file.size), Style::default().fg(Color::Yellow)),
            Span::styled("  Mode: ", Style::default().fg(Color::Gray)),
            Span::styled(&app.file.mode, Style::default().fg(Color::Green)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title(" File "));

    frame.render_widget(info, area);
}

fn render_attrs(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Namespace", "Name", "Value", "Size"]
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
        .attrs
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, attr)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let ns_color = match attr.namespace.as_str() {
                "user" => Color::Cyan,
                "security" => Color::Red,
                "trusted" => Color::Yellow,
                "system" => Color::Magenta,
                _ => Color::White,
            };

            Row::new(vec![
                Cell::from(attr.namespace.clone()).style(Style::default().fg(ns_color)),
                Cell::from(attr.name.clone()),
                Cell::from(truncate(&attr.value, 40)),
                Cell::from(format!("{} B", attr.size)).style(Style::default().fg(Color::DarkGray)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(12),
        Constraint::Percentage(25),
        Constraint::Percentage(45),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Attributes ({}/{}) ", app.selected + 1, app.attrs.len())),
        );

    frame.render_widget(table, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max-3])
    } else {
        s.to_string()
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
