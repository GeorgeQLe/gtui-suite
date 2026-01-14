use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("SFTP CLIENT", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{}@{}", app.username, app.host), Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::styled(&app.current_path, Style::default().fg(Color::Yellow)),
    ])).block(Block::default().borders(Borders::ALL).title(" SSH File Transfer Protocol "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.files.iter().enumerate().map(|(i, f)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let name_style = if f.is_dir { Style::default().fg(Color::Blue) } else if f.permissions.starts_with("-rwx") { Style::default().fg(Color::Green) } else { Style::default() };
        let size_str = if f.is_dir { "-".to_string() } else { format_size(f.size) };
        Row::new(vec![
            Cell::from(f.permissions.clone()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(f.owner.clone()),
            Cell::from(size_str),
            Cell::from(f.modified.clone()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(f.name.clone()).style(name_style),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Length(12), Constraint::Length(10), Constraint::Length(10), Constraint::Length(12), Constraint::Percentage(50)])
        .header(Row::new(["Permissions", "Owner", "Size", "Modified", "Name"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Files ({}/{}) ", app.selected + 1, app.files.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{}B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1}K", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0)) }
    else { format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0)) }
}
