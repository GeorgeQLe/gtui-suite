use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::{App, Protocol};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let filtered = app.filtered_packets();
    let status = if app.is_capturing { "CAPTURING" } else { "STOPPED" };
    let header = Paragraph::new(Line::from(vec![
        Span::styled("PACKET ANALYZER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} packets", filtered.len()), Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(status, Style::default().fg(if app.is_capturing { Color::Green } else { Color::Red })),
    ])).block(Block::default().borders(Borders::ALL).title(" Network Packet Capture "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = filtered.iter().enumerate().map(|(i, p)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let proto_str = match p.protocol {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
            Protocol::Icmp => "ICMP",
            Protocol::Arp => "ARP",
            Protocol::Dns => "DNS",
            Protocol::Http => "HTTP",
            Protocol::Https => "HTTPS",
            Protocol::Unknown => "???",
        };
        let proto_color = match p.protocol {
            Protocol::Tcp | Protocol::Https => Color::Cyan,
            Protocol::Udp => Color::Blue,
            Protocol::Icmp => Color::Yellow,
            Protocol::Dns => Color::Magenta,
            Protocol::Http => Color::Green,
            _ => Color::White,
        };
        let src = p.src_port.map(|port| format!("{}:{}", p.src_ip, port)).unwrap_or_else(|| p.src_ip.clone());
        let dst = p.dst_port.map(|port| format!("{}:{}", p.dst_ip, port)).unwrap_or_else(|| p.dst_ip.clone());

        Row::new(vec![
            Cell::from(format!("{}", p.id)),
            Cell::from(p.timestamp.format("%H:%M:%S%.3f").to_string()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(proto_str).style(Style::default().fg(proto_color)),
            Cell::from(src),
            Cell::from(dst),
            Cell::from(format!("{}", p.length)),
            Cell::from(p.info.clone()),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(6), Constraint::Length(14), Constraint::Length(6),
        Constraint::Percentage(20), Constraint::Percentage(20),
        Constraint::Length(8), Constraint::Percentage(25)
    ])
        .header(Row::new(["#", "Time", "Proto", "Source", "Destination", "Length", "Info"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Packets ({}/{}) ", app.selected + 1, filtered.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
