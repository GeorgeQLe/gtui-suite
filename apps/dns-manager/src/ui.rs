use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs}};
use crate::app::{App, RecordType};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let status_color = if app.service_status == "running" { Color::Green } else { Color::Red };
    let header = Paragraph::new(Line::from(vec![
        Span::styled("DNS MANAGER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | Service: "),
        Span::styled(&app.service_status, Style::default().fg(status_color)),
        Span::raw(format!(" | {} zones", app.zones.len())),
    ])).block(Block::default().borders(Borders::ALL).title(" dnsmasq Configuration "));
    frame.render_widget(header, chunks[0]);

    let zone_names: Vec<Line> = app.zones.iter().map(|z| Line::from(z.name.clone())).collect();
    let tabs = Tabs::new(zone_names)
        .block(Block::default().borders(Borders::ALL).title(" Zones "))
        .select(app.selected_zone)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, chunks[1]);

    if let Some(zone) = app.current_zone() {
        let rows: Vec<Row> = zone.records.iter().enumerate().map(|(i, r)| {
            let style = if i == app.selected_record { Style::default().bg(Color::DarkGray) } else { Style::default() };
            let type_str = match r.record_type {
                RecordType::A => "A",
                RecordType::AAAA => "AAAA",
                RecordType::CNAME => "CNAME",
                RecordType::MX => "MX",
                RecordType::TXT => "TXT",
                RecordType::NS => "NS",
                RecordType::PTR => "PTR",
                RecordType::SRV => "SRV",
            };
            let status = if r.enabled { "[x]" } else { "[ ]" };
            let status_color = if r.enabled { Color::Green } else { Color::DarkGray };
            Row::new(vec![
                Cell::from(status).style(Style::default().fg(status_color)),
                Cell::from(r.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(type_str).style(Style::default().fg(Color::Magenta)),
                Cell::from(r.value.clone()),
                Cell::from(format!("{}", r.ttl)),
            ]).style(style)
        }).collect();

        let table = Table::new(rows, [
            Constraint::Length(5), Constraint::Percentage(20), Constraint::Length(8),
            Constraint::Percentage(40), Constraint::Length(8)
        ])
            .header(Row::new(["", "Name", "Type", "Value", "TTL"]).style(Style::default().fg(Color::Yellow)))
            .block(Block::default().borders(Borders::ALL).title(format!(" {} Records ({}/{}) ", zone.name, app.selected_record + 1, zone.records.len())));
        frame.render_widget(table, chunks[2]);
    }

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[3]);
}
