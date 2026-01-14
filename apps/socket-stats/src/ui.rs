use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::{App, SocketState, SocketType};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let filtered = app.filtered_sockets();
    let header = Paragraph::new(Line::from(vec![
        Span::styled("SOCKET STATS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} sockets", filtered.len()), Style::default().fg(Color::Yellow)),
    ])).block(Block::default().borders(Borders::ALL).title(" ss - Socket Statistics "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = filtered.iter().enumerate().map(|(i, s)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let type_str = match s.socket_type {
            SocketType::Tcp => "tcp",
            SocketType::Udp => "udp",
            SocketType::Unix => "unix",
            SocketType::Raw => "raw",
        };
        let state_str = match s.state {
            SocketState::Listen => "LISTEN",
            SocketState::Established => "ESTAB",
            SocketState::TimeWait => "TIME-WAIT",
            SocketState::CloseWait => "CLOSE-WAIT",
            SocketState::SynSent => "SYN-SENT",
            SocketState::FinWait => "FIN-WAIT",
            SocketState::Closing => "CLOSING",
            SocketState::Unknown => "",
        };
        let state_color = match s.state {
            SocketState::Established => Color::Green,
            SocketState::Listen => Color::Cyan,
            SocketState::TimeWait => Color::Yellow,
            _ => Color::DarkGray,
        };
        let queue_color = if s.recv_q > 0 || s.send_q > 0 { Color::Yellow } else { Color::DarkGray };

        Row::new(vec![
            Cell::from(type_str).style(Style::default().fg(Color::Magenta)),
            Cell::from(state_str).style(Style::default().fg(state_color)),
            Cell::from(format!("{}", s.recv_q)).style(Style::default().fg(queue_color)),
            Cell::from(format!("{}", s.send_q)).style(Style::default().fg(queue_color)),
            Cell::from(s.local_addr.clone()),
            Cell::from(s.peer_addr.clone()),
            Cell::from(s.process.clone().unwrap_or_default()).style(Style::default().fg(Color::Cyan)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(6), Constraint::Length(12), Constraint::Length(8), Constraint::Length(8),
        Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(15)
    ])
        .header(Row::new(["Type", "State", "Recv-Q", "Send-Q", "Local Address", "Peer Address", "Process"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Sockets ({}/{}) ", app.selected + 1, filtered.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
