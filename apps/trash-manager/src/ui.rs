use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_list(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "TRASH MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} items", app.items.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Total: {}", format_size(app.total_size)),
            Style::default().fg(Color::Red),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Recycle Bin "));

    frame.render_widget(header, area);
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    if app.items.is_empty() {
        let empty = Paragraph::new("Trash is empty")
            .block(Block::default().borders(Borders::ALL).title(" Items "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let icon = if item.is_dir { "📁" } else { "📄" };

            let line = Line::from(vec![
                Span::raw(format!("{} ", icon)),
                Span::styled(&item.name, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", item.size_formatted()),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(" "),
                Span::styled(
                    item.deleted_at.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(Color::Blue),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Deleted Items "),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
