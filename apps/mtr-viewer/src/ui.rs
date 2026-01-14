use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let status = if app.is_running { "RUNNING" } else { "PAUSED" };
    let header = Paragraph::new(Line::from(vec![
        Span::styled("MTR VIEWER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | Target: "),
        Span::styled(&app.target, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(status, Style::default().fg(if app.is_running { Color::Green } else { Color::Red })),
    ])).block(Block::default().borders(Borders::ALL).title(" Network Path Analysis "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.hops.iter().enumerate().map(|(i, hop)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let host_str = hop.host.clone().or_else(|| hop.ip.clone()).unwrap_or_else(|| "???".into());
        let ip_str = hop.ip.clone().unwrap_or_default();
        let loss_color = if hop.loss_percent > 50.0 { Color::Red } else if hop.loss_percent > 5.0 { Color::Yellow } else { Color::Green };
        let latency_str = hop.last_ms.map(|l| format!("{:.1}", l)).unwrap_or_else(|| "-".into());

        Row::new(vec![
            Cell::from(format!("{}", hop.number)).style(Style::default().fg(Color::Cyan)),
            Cell::from(host_str),
            Cell::from(ip_str).style(Style::default().fg(Color::DarkGray)),
            Cell::from(format!("{:.1}%", hop.loss_percent)).style(Style::default().fg(loss_color)),
            Cell::from(format!("{}/{}", hop.received, hop.sent)),
            Cell::from(latency_str),
            Cell::from(format!("{:.1}", hop.avg_ms)),
            Cell::from(format!("{:.1}", hop.best_ms)).style(Style::default().fg(Color::Green)),
            Cell::from(format!("{:.1}", hop.worst_ms)).style(Style::default().fg(Color::Red)),
            Cell::from(format!("{:.1}", hop.stdev_ms)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(4), Constraint::Percentage(20), Constraint::Percentage(15),
        Constraint::Length(8), Constraint::Length(10), Constraint::Length(8),
        Constraint::Length(8), Constraint::Length(8), Constraint::Length(8), Constraint::Length(8)
    ])
        .header(Row::new(["#", "Host", "IP", "Loss%", "Snt/Rcv", "Last", "Avg", "Best", "Wrst", "StDev"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Route ({} hops) ", app.hops.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
