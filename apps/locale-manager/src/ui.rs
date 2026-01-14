use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("LOCALE MANAGER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | Default: "),
        Span::styled(&app.current_locale, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ])).block(Block::default().borders(Borders::ALL).title(" /etc/locale.gen "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.locales.iter().enumerate().map(|(i, l)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let status = if l.enabled { "[x]" } else { "[ ]" };
        let default_marker = if l.code == app.current_locale { " *" } else { "" };
        Row::new(vec![
            Cell::from(status).style(Style::default().fg(if l.enabled { Color::Green } else { Color::DarkGray })),
            Cell::from(l.code.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(l.name.clone()),
            Cell::from(l.charset.clone()).style(Style::default().fg(Color::Yellow)),
            Cell::from(default_marker).style(Style::default().fg(Color::Magenta)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Length(5), Constraint::Length(15), Constraint::Percentage(40), Constraint::Length(10), Constraint::Length(3)])
        .header(Row::new(["", "Code", "Name", "Charset", ""]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Locales ({}/{}) ", app.selected + 1, app.locales.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
