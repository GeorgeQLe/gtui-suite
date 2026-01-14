use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("TIMEZONE SELECTOR", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | Current: "),
        Span::styled(&app.current_tz, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ])).block(Block::default().borders(Borders::ALL).title(" /etc/localtime "));
    frame.render_widget(header, chunks[0]);

    let filtered = app.filtered_timezones();
    let rows: Vec<Row> = filtered.iter().enumerate().map(|(i, (_, tz))| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let full_tz = format!("{}/{}", tz.region, tz.city);
        let is_current = full_tz == app.current_tz;
        Row::new(vec![
            Cell::from(if is_current { "*" } else { "" }).style(Style::default().fg(Color::Green)),
            Cell::from(tz.region.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(tz.city.clone()),
            Cell::from(tz.offset.clone()).style(Style::default().fg(Color::Yellow)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Length(3), Constraint::Length(15), Constraint::Percentage(50), Constraint::Length(10)])
        .header(Row::new(["", "Region", "City", "Offset"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Timezones ({}/{}) ", app.selected + 1, filtered.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
