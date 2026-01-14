use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("RDP MANAGER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} connections", app.connections.len()), Style::default().fg(Color::Yellow)),
    ])).block(Block::default().borders(Borders::ALL).title(" Remote Desktop Connections "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.connections.iter().enumerate().map(|(i, c)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let fs = if c.fullscreen { "FS" } else { "W" };
        let domain = c.domain.clone().unwrap_or_else(|| "-".into());
        let last = c.last_connected.clone().unwrap_or_else(|| "Never".into());
        Row::new(vec![
            Cell::from(c.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{}:{}", c.host, c.port)),
            Cell::from(c.username.clone()),
            Cell::from(domain),
            Cell::from(c.resolution.clone()),
            Cell::from(fs).style(Style::default().fg(if c.fullscreen { Color::Green } else { Color::DarkGray })),
            Cell::from(last).style(Style::default().fg(Color::DarkGray)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Percentage(15), Constraint::Percentage(25), Constraint::Length(12), Constraint::Length(10), Constraint::Length(12), Constraint::Length(4), Constraint::Length(12)])
        .header(Row::new(["Name", "Host", "User", "Domain", "Resolution", "FS", "Last"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Connections ({}/{}) ", app.selected + 1, app.connections.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
