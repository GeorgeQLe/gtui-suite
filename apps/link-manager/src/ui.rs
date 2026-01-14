use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{App, LinkStatus, LinkType};

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
    let broken = app.broken_count();

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "LINK MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} symlinks", app.symlink_count()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} hardlinks", app.hardlink_count()),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} broken", broken),
            Style::default().fg(if broken > 0 { Color::Red } else { Color::Green }),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Symlink & Hardlink Manager "));

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Type", "Path", "Target", "Status", "Links"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1);

    let filtered = app.filtered_links();
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
        .map(|(i, link)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let type_color = match link.link_type {
                LinkType::Symlink => Color::Cyan,
                LinkType::Hardlink => Color::Magenta,
            };

            let status_color = match link.status {
                LinkStatus::Valid => Color::Green,
                LinkStatus::Broken => Color::Red,
                LinkStatus::Circular => Color::Yellow,
            };

            Row::new(vec![
                Cell::from(link.link_type.name()).style(Style::default().fg(type_color)),
                Cell::from(truncate(&link.path, 30)),
                Cell::from(truncate(&link.target, 30)),
                Cell::from(link.status.name()).style(Style::default().fg(status_color)),
                Cell::from(link.link_count.to_string()).style(Style::default().fg(Color::DarkGray)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Percentage(35),
        Constraint::Percentage(35),
        Constraint::Length(10),
        Constraint::Length(6),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Links ({}/{}) ", app.selected + 1, filtered.len())),
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
