use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("FSTAB EDITOR", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} entries", app.entries.len()), Style::default().fg(Color::Yellow)),
    ])).block(Block::default().borders(Borders::ALL).title(" /etc/fstab "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.entries.iter().enumerate().map(|(i, e)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        Row::new(vec![
            Cell::from(e.device.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(e.mount_point.clone()),
            Cell::from(e.fs_type.clone()).style(Style::default().fg(Color::Yellow)),
            Cell::from(e.options.clone()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(format!("{} {}", e.dump, e.pass)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Length(10), Constraint::Percentage(35), Constraint::Length(8)])
        .header(Row::new(["Device", "Mount", "Type", "Options", "D P"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Entries ({}/{}) ", app.selected + 1, app.entries.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
