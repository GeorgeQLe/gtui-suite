use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap}};
use crate::app::{App, ViewMode};

pub fn render(frame: &mut Frame, app: &App) {
    match app.view_mode {
        ViewMode::List => render_list(frame, app),
        ViewMode::Detail => render_detail(frame, app),
    }
}

fn render_list(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("WHOIS BROWSER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} queries", app.history.len()), Style::default().fg(Color::Yellow)),
    ])).block(Block::default().borders(Borders::ALL).title(" Domain/IP Lookup History "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.history.iter().enumerate().map(|(i, r)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let registrar = r.registrar.clone().unwrap_or_else(|| "N/A".into());
        let expires = r.expiration_date.clone().unwrap_or_else(|| "N/A".into());

        Row::new(vec![
            Cell::from(r.query.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(registrar),
            Cell::from(r.creation_date.clone().unwrap_or_default()),
            Cell::from(expires),
            Cell::from(r.timestamp.format("%Y-%m-%d %H:%M").to_string()).style(Style::default().fg(Color::DarkGray)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Length(12),
        Constraint::Length(12), Constraint::Length(18)
    ])
        .header(Row::new(["Domain/IP", "Registrar", "Created", "Expires", "Queried"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" History ({}/{}) ", app.selected + 1, app.history.len().max(1))));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}

fn render_detail(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    if let Some(result) = app.history.get(app.selected) {
        let header = Paragraph::new(Line::from(vec![
            Span::styled("WHOIS BROWSER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" | "),
            Span::styled(&result.query, Style::default().fg(Color::Yellow)),
        ])).block(Block::default().borders(Borders::ALL).title(" WHOIS Result "));
        frame.render_widget(header, chunks[0]);

        let mut content = vec![
            format!("Query: {}", result.query),
            format!("Registrar: {}", result.registrar.clone().unwrap_or_else(|| "N/A".into())),
            format!("Created: {}", result.creation_date.clone().unwrap_or_else(|| "N/A".into())),
            format!("Expires: {}", result.expiration_date.clone().unwrap_or_else(|| "N/A".into())),
            String::new(),
            "Name Servers:".to_string(),
        ];
        for ns in &result.name_servers {
            content.push(format!("  {}", ns));
        }
        content.push(String::new());
        content.push("Status:".to_string());
        for status in &result.status {
            content.push(format!("  {}", status));
        }
        content.push(String::new());
        content.push("Raw Data:".to_string());
        content.push(result.raw_data.clone());

        let para = Paragraph::new(content.join("\n"))
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: false })
            .scroll((app.scroll_offset as u16, 0));
        frame.render_widget(para, chunks[1]);
    }

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
