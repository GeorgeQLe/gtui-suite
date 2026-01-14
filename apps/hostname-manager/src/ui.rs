use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("HOSTNAME MANAGER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | Hostname: "),
        Span::styled(&app.hostname, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ])).block(Block::default().borders(Borders::ALL).title(" /etc/hostname & /etc/hosts "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.hosts.iter().enumerate().map(|(i, e)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        Row::new(vec![
            Cell::from(e.ip.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(e.hostnames.join(", ")),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Length(20), Constraint::Percentage(80)])
        .header(Row::new(["IP Address", "Hostnames"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Hosts ({}/{}) ", app.selected + 1, app.hosts.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
