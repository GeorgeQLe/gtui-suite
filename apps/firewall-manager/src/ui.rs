use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs}};
use crate::app::{App, Chain, Protocol, RuleAction};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let fw_status = if app.firewall_enabled { ("ENABLED", Color::Green) } else { ("DISABLED", Color::Red) };
    let header = Paragraph::new(Line::from(vec![
        Span::styled("FIREWALL MANAGER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | Status: "),
        Span::styled(fw_status.0, Style::default().fg(fw_status.1)),
        Span::raw(format!(" | {} rules", app.rules.len())),
    ])).block(Block::default().borders(Borders::ALL).title(" iptables/nftables/ufw "));
    frame.render_widget(header, chunks[0]);

    let chains = vec![
        Line::from("INPUT"),
        Line::from("OUTPUT"),
        Line::from("FORWARD"),
    ];
    let chain_idx = match app.chain {
        Chain::Input => 0,
        Chain::Output => 1,
        Chain::Forward => 2,
    };
    let tabs = Tabs::new(chains)
        .block(Block::default().borders(Borders::ALL).title(" Chain "))
        .select(chain_idx)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, chunks[1]);

    let filtered = app.filtered_rules();
    let rows: Vec<Row> = filtered.iter().enumerate().map(|(i, r)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let status = if r.enabled { "[x]" } else { "[ ]" };
        let status_color = if r.enabled { Color::Green } else { Color::DarkGray };
        let action_str = match r.action {
            RuleAction::Allow => "ALLOW",
            RuleAction::Deny => "DENY",
            RuleAction::Reject => "REJECT",
            RuleAction::Drop => "DROP",
        };
        let action_color = match r.action {
            RuleAction::Allow => Color::Green,
            RuleAction::Deny | RuleAction::Reject | RuleAction::Drop => Color::Red,
        };
        let proto_str = match r.protocol {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
            Protocol::Icmp => "ICMP",
            Protocol::Any => "ANY",
        };
        let port_str = r.port.clone().unwrap_or_else(|| "*".into());
        let comment = r.comment.clone().unwrap_or_default();

        Row::new(vec![
            Cell::from(status).style(Style::default().fg(status_color)),
            Cell::from(action_str).style(Style::default().fg(action_color)),
            Cell::from(proto_str).style(Style::default().fg(Color::Magenta)),
            Cell::from(r.source.clone()),
            Cell::from(port_str),
            Cell::from(comment).style(Style::default().fg(Color::DarkGray)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(5), Constraint::Length(8), Constraint::Length(6),
        Constraint::Percentage(25), Constraint::Length(12), Constraint::Percentage(30)
    ])
        .header(Row::new(["", "Action", "Proto", "Source", "Port", "Comment"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Rules ({}/{}) ", app.selected + 1, filtered.len().max(1))));
    frame.render_widget(table, chunks[2]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[3]);
}
