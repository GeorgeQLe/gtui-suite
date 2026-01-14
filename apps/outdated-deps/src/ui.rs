use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{App, UpdateType};

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
    let major = app.major_count();

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "OUTDATED DEPS",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} outdated", app.deps.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} major", major),
            Style::default().fg(if major > 0 { Color::Red } else { Color::Green }),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} selected", app.selected_count()),
            Style::default().fg(Color::Green),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Dependency Update Manager "));

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["", "Package", "Current", "Latest", "Type", "Breaking"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1);

    let filtered = app.filtered_deps();
    let visible_height = area.height.saturating_sub(3) as usize;
    let start = if app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, dep)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let checkbox = if dep.selected { "[x]" } else { "[ ]" };
            let checkbox_color = if dep.selected { Color::Green } else { Color::DarkGray };

            let type_color = match dep.update_type {
                UpdateType::Patch => Color::Green,
                UpdateType::Minor => Color::Yellow,
                UpdateType::Major => Color::Red,
            };

            Row::new(vec![
                Cell::from(checkbox).style(Style::default().fg(checkbox_color)),
                Cell::from(dep.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(dep.current.clone()),
                Cell::from(dep.latest.clone()).style(Style::default().fg(Color::Green)),
                Cell::from(dep.update_type.name()).style(Style::default().fg(type_color)),
                Cell::from(if dep.has_breaking { "Yes" } else { "-" })
                    .style(Style::default().fg(if dep.has_breaking { Color::Red } else { Color::DarkGray })),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Percentage(25),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Length(8),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Dependencies ({}/{}) ", app.selected + 1, filtered.len())),
        );

    frame.render_widget(table, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
