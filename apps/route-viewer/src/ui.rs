use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::{App, RouteType};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("ROUTE VIEWER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} routes", app.routes.len()), Style::default().fg(Color::Yellow)),
        Span::raw(if app.show_ipv6 { " [IPv4+IPv6]" } else { " [IPv4]" }),
    ])).block(Block::default().borders(Borders::ALL).title(" Kernel IP Routing Table "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.routes.iter().enumerate().map(|(i, r)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let type_str = match r.route_type {
            RouteType::Local => "LOCAL",
            RouteType::Gateway => "GW",
            RouteType::Host => "HOST",
            RouteType::Default => "DEFAULT",
        };
        let type_color = match r.route_type {
            RouteType::Default => Color::Green,
            RouteType::Gateway => Color::Cyan,
            RouteType::Local => Color::Yellow,
            RouteType::Host => Color::Magenta,
        };
        Row::new(vec![
            Cell::from(r.destination.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(r.gateway.clone()),
            Cell::from(r.netmask.clone()),
            Cell::from(r.flags.clone()),
            Cell::from(format!("{}", r.metric)),
            Cell::from(r.interface.clone()).style(Style::default().fg(Color::Yellow)),
            Cell::from(type_str).style(Style::default().fg(type_color)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(18), Constraint::Percentage(15), Constraint::Percentage(18),
        Constraint::Length(8), Constraint::Length(8), Constraint::Length(10), Constraint::Length(10)
    ])
        .header(Row::new(["Destination", "Gateway", "Netmask", "Flags", "Metric", "Iface", "Type"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Routes ({}/{}) ", app.selected + 1, app.routes.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
