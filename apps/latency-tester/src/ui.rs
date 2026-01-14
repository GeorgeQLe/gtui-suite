use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Sparkline, Table}};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(5), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("LATENCY TESTER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} endpoints", app.endpoints.len()), Style::default().fg(Color::Yellow)),
        Span::raw(if app.auto_test { " [AUTO]" } else { "" }),
    ])).block(Block::default().borders(Borders::ALL).title(" Network Latency Monitor "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.endpoints.iter().enumerate().map(|(i, ep)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let latency_str = ep.latency_ms.map(|l| format!("{:.1}ms", l)).unwrap_or_else(|| "timeout".into());
        let latency_color = match ep.latency_ms {
            Some(l) if l < 20.0 => Color::Green,
            Some(l) if l < 100.0 => Color::Yellow,
            Some(_) => Color::Red,
            None => Color::DarkGray,
        };
        let loss_color = if ep.packet_loss > 50.0 { Color::Red } else if ep.packet_loss > 0.0 { Color::Yellow } else { Color::Green };
        Row::new(vec![
            Cell::from(ep.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(ep.host.clone()),
            Cell::from(latency_str).style(Style::default().fg(latency_color)),
            Cell::from(format!("{:.1}ms", ep.min_ms)),
            Cell::from(format!("{:.1}ms", ep.max_ms)),
            Cell::from(format!("{:.1}ms", ep.avg_ms)),
            Cell::from(format!("{:.1}%", ep.packet_loss)).style(Style::default().fg(loss_color)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Percentage(15), Constraint::Percentage(25), Constraint::Length(12), Constraint::Length(10), Constraint::Length(10), Constraint::Length(10), Constraint::Length(8)])
        .header(Row::new(["Name", "Host", "Current", "Min", "Max", "Avg", "Loss"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Endpoints ({}/{}) ", app.selected + 1, app.endpoints.len())));
    frame.render_widget(table, chunks[1]);

    // Show history sparkline for selected endpoint
    if let Some(ep) = app.endpoints.get(app.selected) {
        let history_data: Vec<u64> = ep.history.iter().map(|&v| (v * 10.0) as u64).collect();
        let sparkline = Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title(format!(" {} History ", ep.name)))
            .data(&history_data)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(sparkline, chunks[2]);
    }

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[3]);
}
