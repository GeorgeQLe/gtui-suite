use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::{App, ViewMode};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let view_name = match app.view_mode {
        ViewMode::Registries => "Registries".to_string(),
        ViewMode::Repos => app.registries.get(app.current_registry).map(|r| r.name.clone()).unwrap_or_default(),
        ViewMode::Tags => app.current_repo.clone().unwrap_or_default(),
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled("REGISTRY BROWSER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(view_name, Style::default().fg(Color::Yellow)),
    ])).block(Block::default().borders(Borders::ALL).title(" Container Registry "));
    frame.render_widget(header, chunks[0]);

    match app.view_mode {
        ViewMode::Registries => render_registries(frame, chunks[1], app),
        ViewMode::Repos => render_repos(frame, chunks[1], app),
        ViewMode::Tags => render_tags(frame, chunks[1], app),
    }

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}

fn render_registries(frame: &mut Frame, area: Rect, app: &App) {
    let rows: Vec<Row> = app.registries.iter().enumerate().map(|(i, r)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let auth = if r.authenticated { "Yes" } else { "No" };
        Row::new(vec![
            Cell::from(r.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(r.url.clone()),
            Cell::from(auth).style(Style::default().fg(if r.authenticated { Color::Green } else { Color::Red })),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Percentage(25), Constraint::Percentage(55), Constraint::Length(10)])
        .header(Row::new(["Name", "URL", "Auth"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Registries ({}/{}) ", app.selected + 1, app.registries.len())));
    frame.render_widget(table, area);
}

fn render_repos(frame: &mut Frame, area: Rect, app: &App) {
    let rows: Vec<Row> = app.repos.iter().enumerate().map(|(i, r)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        Row::new(vec![
            Cell::from(r.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{}", r.tag_count)),
            Cell::from(r.last_updated.clone()).style(Style::default().fg(Color::DarkGray)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Percentage(60), Constraint::Length(10), Constraint::Length(15)])
        .header(Row::new(["Repository", "Tags", "Updated"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Repositories ({}/{}) ", app.selected + 1, app.repos.len())));
    frame.render_widget(table, area);
}

fn render_tags(frame: &mut Frame, area: Rect, app: &App) {
    let rows: Vec<Row> = app.tags.iter().enumerate().map(|(i, t)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        Row::new(vec![
            Cell::from(t.name.clone()).style(Style::default().fg(if t.name == "latest" { Color::Green } else { Color::Cyan })),
            Cell::from(&t.digest[7..19]),
            Cell::from(format_size(t.size)),
            Cell::from(t.pushed.clone()).style(Style::default().fg(Color::DarkGray)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Percentage(30), Constraint::Length(14), Constraint::Length(10), Constraint::Length(15)])
        .header(Row::new(["Tag", "Digest", "Size", "Pushed"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Tags ({}/{}) ", app.selected + 1, app.tags.len())));
    frame.render_widget(table, area);
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 * 1024 { format!("{:.1}KB", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0)) }
    else { format!("{:.2}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0)) }
}
