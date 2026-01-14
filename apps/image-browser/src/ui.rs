use ratatui::{prelude::*, widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table}};
use crate::app::{App, ViewMode};

pub fn render(frame: &mut Frame, app: &App) {
    match app.view_mode {
        ViewMode::List => render_list(frame, app),
        ViewMode::Layers => render_layers(frame, app),
    }
}

fn render_list(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let total_size: u64 = app.images.iter().map(|i| i.size).sum();
    let header = Paragraph::new(Line::from(vec![
        Span::styled("IMAGE BROWSER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} images", app.images.len()), Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(format_size(total_size), Style::default().fg(Color::Magenta)),
    ])).block(Block::default().borders(Borders::ALL).title(" Container Images "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.images.iter().enumerate().map(|(i, img)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        Row::new(vec![
            Cell::from(img.repository.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(img.tag.clone()).style(Style::default().fg(Color::Yellow)),
            Cell::from(&img.id[7..19]),
            Cell::from(format_size(img.size)),
            Cell::from(img.created.clone()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(format!("{} layers", img.layers.len())),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Percentage(25), Constraint::Length(15), Constraint::Length(14), Constraint::Length(10), Constraint::Length(15), Constraint::Length(10)])
        .header(Row::new(["Repository", "Tag", "Image ID", "Size", "Created", "Layers"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Images ({}/{}) ", app.selected + 1, app.images.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}

fn render_layers(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    if let Some(img) = app.images.get(app.selected) {
        let header = Paragraph::new(Line::from(vec![
            Span::styled("IMAGE BROWSER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" | "),
            Span::styled(format!("{}:{}", img.repository, img.tag), Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled(format!("{} layers", img.layers.len()), Style::default().fg(Color::Magenta)),
        ])).block(Block::default().borders(Borders::ALL).title(" Image Layers "));
        frame.render_widget(header, chunks[0]);

        let layer_items: Vec<ListItem> = img.layers.iter().enumerate().map(|(i, layer)| {
            let style = if i == app.layer_scroll { Style::default().bg(Color::DarkGray) } else { Style::default() };
            let size_str = if layer.size > 0 { format_size(layer.size) } else { "0B".into() };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:2}. ", i + 1), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>8} ", size_str), Style::default().fg(Color::Yellow)),
                Span::raw(&layer.command),
            ])).style(style)
        }).collect();

        let list = List::new(layer_items)
            .block(Block::default().borders(Borders::ALL).title(format!(" Layers ({}/{}) ", app.layer_scroll + 1, img.layers.len())));
        frame.render_widget(list, chunks[1]);
    }

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{}B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1}KB", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0)) }
    else { format!("{:.2}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0)) }
}
