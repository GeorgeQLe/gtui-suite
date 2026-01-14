use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::{App, TunnelStatus, TunnelType};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let active_count = app.tunnels.iter().filter(|t| t.status == TunnelStatus::Active).count();
    let header = Paragraph::new(Line::from(vec![
        Span::styled("SSH TUNNEL MANAGER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} tunnels", app.tunnels.len()), Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(format!("{} active", active_count), Style::default().fg(Color::Green)),
    ])).block(Block::default().borders(Borders::ALL).title(" Port Forwarding "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.tunnels.iter().enumerate().map(|(i, t)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let type_str = match t.tunnel_type {
            TunnelType::Local => "LOCAL",
            TunnelType::Remote => "REMOTE",
            TunnelType::Dynamic => "SOCKS",
        };
        let (status_str, status_color) = match t.status {
            TunnelStatus::Active => ("ACTIVE", Color::Green),
            TunnelStatus::Inactive => ("INACTIVE", Color::DarkGray),
            TunnelStatus::Connecting => ("CONNECTING", Color::Yellow),
            TunnelStatus::Error => ("ERROR", Color::Red),
        };
        let forward_str = match t.tunnel_type {
            TunnelType::Dynamic => format!("localhost:{}", t.local_port),
            TunnelType::Local => format!("localhost:{} -> {}:{}", t.local_port, t.remote_host, t.remote_port),
            TunnelType::Remote => format!("{}:{} -> localhost:{}", t.remote_host, t.remote_port, t.local_port),
        };
        let reconnect = if t.auto_reconnect { "R" } else { "-" };

        Row::new(vec![
            Cell::from(status_str).style(Style::default().fg(status_color)),
            Cell::from(t.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(type_str).style(Style::default().fg(Color::Magenta)),
            Cell::from(format!("{}@{}", t.ssh_user, t.ssh_host)),
            Cell::from(forward_str),
            Cell::from(reconnect).style(Style::default().fg(if t.auto_reconnect { Color::Green } else { Color::DarkGray })),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Length(12), Constraint::Percentage(15), Constraint::Length(8), Constraint::Percentage(20), Constraint::Percentage(35), Constraint::Length(3)])
        .header(Row::new(["Status", "Name", "Type", "SSH Host", "Forward", "R"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Tunnels ({}/{}) ", app.selected + 1, app.tunnels.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
