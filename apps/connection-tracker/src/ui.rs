use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::{App, ConnectionState, Protocol};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let filtered = app.filtered_connections();
    let header = Paragraph::new(Line::from(vec![
        Span::styled("CONNECTION TRACKER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} connections", filtered.len()), Style::default().fg(Color::Yellow)),
    ])).block(Block::default().borders(Borders::ALL).title(" Active Network Connections "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = filtered.iter().enumerate().map(|(i, c)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let proto_str = match c.protocol {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
            Protocol::Tcp6 => "TCP6",
            Protocol::Udp6 => "UDP6",
        };
        let state_str = match c.state {
            ConnectionState::Established => "ESTAB",
            ConnectionState::Listen => "LISTEN",
            ConnectionState::TimeWait => "TIME_WAIT",
            ConnectionState::CloseWait => "CLOSE_WAIT",
            ConnectionState::SynSent => "SYN_SENT",
            ConnectionState::SynRecv => "SYN_RECV",
            ConnectionState::FinWait1 => "FIN_WAIT1",
            ConnectionState::FinWait2 => "FIN_WAIT2",
            ConnectionState::Closing => "CLOSING",
            ConnectionState::LastAck => "LAST_ACK",
            ConnectionState::Closed => "CLOSED",
        };
        let state_color = match c.state {
            ConnectionState::Established => Color::Green,
            ConnectionState::Listen => Color::Cyan,
            ConnectionState::TimeWait => Color::Yellow,
            _ => Color::DarkGray,
        };
        Row::new(vec![
            Cell::from(proto_str).style(Style::default().fg(Color::Magenta)),
            Cell::from(format!("{}:{}", c.local_addr, c.local_port)),
            Cell::from(format!("{}:{}", c.remote_addr, c.remote_port)),
            Cell::from(state_str).style(Style::default().fg(state_color)),
            Cell::from(c.pid.map(|p| p.to_string()).unwrap_or_default()),
            Cell::from(c.process.clone().unwrap_or_default()).style(Style::default().fg(Color::Cyan)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Length(6), Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Length(12), Constraint::Length(8), Constraint::Percentage(15)])
        .header(Row::new(["Proto", "Local Address", "Remote Address", "State", "PID", "Process"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Connections ({}/{}) ", app.selected + 1, filtered.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
