use ratatui::{prelude::*, widgets::{Block, Borders, List, ListItem, Paragraph, Tabs}};
use crate::app::{App, LogLevel};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let filtered = app.filtered_logs();
    let header = Paragraph::new(Line::from(vec![
        Span::styled("CONTAINER LOGS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} entries", filtered.len()), Style::default().fg(Color::Yellow)),
        Span::raw(if app.follow { " [FOLLOW]" } else { "" }),
    ])).block(Block::default().borders(Borders::ALL).title(" Multi-Container Log Aggregator "));
    frame.render_widget(header, chunks[0]);

    let container_names: Vec<Line> = app.containers.iter().map(|c| {
        let style = if c.enabled { Style::default().fg(Color::Green) } else { Style::default().fg(Color::DarkGray) };
        Line::from(Span::styled(c.name.clone(), style))
    }).collect();
    let tabs = Tabs::new(container_names)
        .block(Block::default().borders(Borders::ALL).title(" Containers "))
        .select(app.selected_container)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, chunks[1]);

    let container_colors: std::collections::HashMap<&str, Color> = app.containers.iter()
        .map(|c| (c.name.as_str(), match c.color {
            1 => Color::Cyan,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Magenta,
            _ => Color::White,
        }))
        .collect();

    let visible_height = chunks[2].height.saturating_sub(2) as usize;
    let start = app.scroll_offset.saturating_sub(visible_height);
    let log_items: Vec<ListItem> = filtered.iter()
        .skip(start)
        .take(visible_height + 1)
        .map(|log| {
            let time = log.timestamp.format("%H:%M:%S").to_string();
            let level_color = match log.level {
                LogLevel::Info => Color::Green,
                LogLevel::Warn => Color::Yellow,
                LogLevel::Error => Color::Red,
                LogLevel::Debug => Color::DarkGray,
            };
            let level_str = match log.level {
                LogLevel::Info => "INFO ",
                LogLevel::Warn => "WARN ",
                LogLevel::Error => "ERROR",
                LogLevel::Debug => "DEBUG",
            };
            let container_color = container_colors.get(log.container.as_str()).copied().unwrap_or(Color::White);

            ListItem::new(Line::from(vec![
                Span::styled(time, Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::styled(format!("{:8}", log.container), Style::default().fg(container_color)),
                Span::raw(" "),
                Span::styled(level_str, Style::default().fg(level_color)),
                Span::raw(" "),
                Span::raw(&log.message),
            ]))
        })
        .collect();

    let list = List::new(log_items)
        .block(Block::default().borders(Borders::ALL).title(format!(" Logs ({}/{}) ", app.scroll_offset + 1, filtered.len().max(1))));
    frame.render_widget(list, chunks[2]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[3]);
}
