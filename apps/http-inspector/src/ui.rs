use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap}};
use crate::app::{App, HttpMethod, ViewMode};

pub fn render(frame: &mut Frame, app: &App) {
    match app.view_mode {
        ViewMode::List => render_list(frame, app),
        ViewMode::RequestDetail | ViewMode::ResponseDetail => render_detail(frame, app),
    }
}

fn render_list(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("HTTP INSPECTOR", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} requests", app.requests.len()), Style::default().fg(Color::Yellow)),
    ])).block(Block::default().borders(Borders::ALL).title(" Request History "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app.requests.iter().enumerate().map(|(i, r)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let method_str = match r.method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DEL",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPT",
        };
        let method_color = match r.method {
            HttpMethod::Get => Color::Green,
            HttpMethod::Post => Color::Yellow,
            HttpMethod::Put => Color::Blue,
            HttpMethod::Delete => Color::Red,
            _ => Color::Magenta,
        };
        let status_color = match r.status {
            200..=299 => Color::Green,
            300..=399 => Color::Yellow,
            400..=499 => Color::Red,
            500..=599 => Color::Magenta,
            _ => Color::White,
        };
        Row::new(vec![
            Cell::from(method_str).style(Style::default().fg(method_color)),
            Cell::from(format!("{}", r.status)).style(Style::default().fg(status_color)),
            Cell::from(r.url.clone()),
            Cell::from(format!("{}ms", r.duration_ms)),
            Cell::from(format_size(r.response_size)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Length(6), Constraint::Length(6), Constraint::Percentage(50), Constraint::Length(10), Constraint::Length(10)])
        .header(Row::new(["Method", "Status", "URL", "Time", "Size"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Requests ({}/{}) ", app.selected + 1, app.requests.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}

fn render_detail(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let view_name = match app.view_mode {
        ViewMode::RequestDetail => "Request",
        ViewMode::ResponseDetail => "Response",
        _ => "Detail",
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(format!("HTTP INSPECTOR - {}", view_name), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ])).block(Block::default().borders(Borders::ALL).title(format!(" {} Detail ", view_name)));
    frame.render_widget(header, chunks[0]);

    if let Some(req) = app.requests.get(app.selected) {
        let content = match app.view_mode {
            ViewMode::RequestDetail => {
                let mut lines = vec![
                    format!("URL: {}", req.url),
                    format!("Method: {:?}", req.method),
                    format!("Time: {}", req.timestamp.format("%Y-%m-%d %H:%M:%S")),
                    String::new(),
                    "Headers:".to_string(),
                ];
                for (k, v) in &req.headers {
                    lines.push(format!("  {}: {}", k, v));
                }
                lines.join("\n")
            },
            ViewMode::ResponseDetail => {
                let mut lines = vec![
                    format!("Status: {}", req.status),
                    format!("Duration: {}ms", req.duration_ms),
                    format!("Size: {} bytes", req.response_size),
                    String::new(),
                    "Body:".to_string(),
                ];
                if let Some(body) = &req.body {
                    lines.push(body.clone());
                }
                lines.join("\n")
            },
            _ => String::new(),
        };

        let para = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: false })
            .scroll((app.scroll_offset as u16, 0));
        frame.render_widget(para, chunks[1]);
    }

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 { format!("{}B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1}KB", bytes as f64 / 1024.0) }
    else { format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0)) }
}
