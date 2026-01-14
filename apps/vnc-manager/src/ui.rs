use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::{App, VncQuality};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("VNC MANAGER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} connections", app.connections.len()), Style::default().fg(Color::Yellow)),
    ])).block(Block::default().borders(Borders::ALL).title(" Virtual Network Computing "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.connections.iter().enumerate().map(|(i, c)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let quality_str = match c.quality {
            VncQuality::Auto => "AUTO",
            VncQuality::Low => "LOW",
            VncQuality::Medium => "MED",
            VncQuality::High => "HIGH",
        };
        let pwd = if c.password_saved { "*" } else { "" };
        let view = if c.view_only { "RO" } else { "RW" };
        let last = c.last_connected.clone().unwrap_or_else(|| "Never".into());
        Row::new(vec![
            Cell::from(c.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{}:{}", c.host, c.port)),
            Cell::from(pwd).style(Style::default().fg(Color::Green)),
            Cell::from(quality_str),
            Cell::from(view).style(Style::default().fg(if c.view_only { Color::Yellow } else { Color::Green })),
            Cell::from(last).style(Style::default().fg(Color::DarkGray)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Percentage(20), Constraint::Percentage(30), Constraint::Length(3), Constraint::Length(6), Constraint::Length(4), Constraint::Length(12)])
        .header(Row::new(["Name", "Host:Port", "Pwd", "Quality", "Mode", "Last"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Connections ({}/{}) ", app.selected + 1, app.connections.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
