use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::{App, DeviceStatus};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let online = app.devices.iter().filter(|d| d.status == DeviceStatus::Online).count();
    let header = Paragraph::new(Line::from(vec![
        Span::styled("WAKE ON LAN", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} devices", app.devices.len()), Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(format!("{} online", online), Style::default().fg(Color::Green)),
    ])).block(Block::default().borders(Borders::ALL).title(" Magic Packet Sender "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.devices.iter().enumerate().map(|(i, d)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let (status_str, status_color) = match d.status {
            DeviceStatus::Online => ("ONLINE", Color::Green),
            DeviceStatus::Offline => ("OFFLINE", Color::Red),
            DeviceStatus::Unknown => ("?", Color::DarkGray),
            DeviceStatus::Waking => ("WAKING", Color::Yellow),
        };
        let ip = d.ip_address.clone().unwrap_or_else(|| "-".into());
        let last = d.last_wake.clone().unwrap_or_else(|| "Never".into());
        Row::new(vec![
            Cell::from(status_str).style(Style::default().fg(status_color)),
            Cell::from(d.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(d.mac_address.clone()),
            Cell::from(ip),
            Cell::from(format!("{}", d.port)),
            Cell::from(last).style(Style::default().fg(Color::DarkGray)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Length(10), Constraint::Percentage(18), Constraint::Length(20), Constraint::Length(16), Constraint::Length(6), Constraint::Percentage(20)])
        .header(Row::new(["Status", "Name", "MAC Address", "IP", "Port", "Last Wake"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Devices ({}/{}) ", app.selected + 1, app.devices.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
