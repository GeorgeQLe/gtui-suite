use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(4), Constraint::Length(1)])
        .split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("BOOTLOADER CONFIG", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} entries", app.entries.len()), Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(format!("Timeout: {}s", app.timeout), Style::default().fg(Color::Green)),
    ])).block(Block::default().borders(Borders::ALL).title(" Boot Configuration "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.entries.iter().enumerate().map(|(i, e)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let def = if e.is_default { "*" } else { " " };
        Row::new(vec![
            Cell::from(def).style(Style::default().fg(Color::Green)),
            Cell::from(e.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(e.kernel.clone()),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Length(3), Constraint::Percentage(40), Constraint::Percentage(50)])
        .header(Row::new(["", "Name", "Kernel"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Boot Entries ({}/{}) ", app.selected + 1, app.entries.len())));
    frame.render_widget(table, chunks[1]);

    if let Some(e) = app.entries.get(app.selected) {
        let detail = Paragraph::new(vec![
            Line::from(vec![Span::styled("Options: ", Style::default().fg(Color::Gray)), Span::raw(&e.options)]),
            Line::from(vec![Span::styled("Initrd: ", Style::default().fg(Color::Gray)), Span::raw(&e.initrd)]),
        ]).block(Block::default().borders(Borders::ALL).title(" Details "));
        frame.render_widget(detail, chunks[2]);
    }

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[3]);
}
