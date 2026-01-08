use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, Mode, RequestSection, View};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    match app.view {
        View::Request => render_request(frame, app, chunks[0]),
        View::Response => render_response(frame, app, chunks[0]),
        View::Collections => render_collections(frame, app, chunks[0]),
        View::History => render_history(frame, app, chunks[0]),
        View::Help => render_help(frame, chunks[0]),
    }

    render_status_bar(frame, app, chunks[1]);

    match &app.mode {
        Mode::EditUrl(text) => render_input_dialog(frame, "URL", text),
        Mode::EditBody(text) => render_text_dialog(frame, "Body", text),
        Mode::AddHeader(key, value) => render_header_dialog(frame, key, value),
        Mode::SaveRequest(name) => render_input_dialog(frame, "Save Request As", name),
        Mode::NewCollection(name) => render_input_dialog(frame, "New Collection", name),
        Mode::EditHeader(_, _) | Mode::Normal => {}
    }
}

fn render_request(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Method + URL
            Constraint::Min(5),    // Headers
            Constraint::Length(5), // Body preview
        ])
        .split(area);

    // Method and URL
    let method_url_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(12), Constraint::Min(1)])
        .split(chunks[0]);

    let method_style = if app.section == RequestSection::Method {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let method_block = Block::default()
        .borders(Borders::ALL)
        .border_style(method_style)
        .title(" Method ");
    let method_text = Paragraph::new(app.current_request.method.as_str())
        .block(method_block)
        .alignment(Alignment::Center);
    frame.render_widget(method_text, method_url_chunks[0]);

    let url_style = if app.section == RequestSection::Url {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let url_block = Block::default()
        .borders(Borders::ALL)
        .border_style(url_style)
        .title(" URL ");
    let url_text = if app.current_request.url.is_empty() {
        "Enter URL (press 'u' to edit)"
    } else {
        &app.current_request.url
    };
    let url = Paragraph::new(url_text).block(url_block);
    frame.render_widget(url, method_url_chunks[1]);

    // Headers
    let headers_style = if app.section == RequestSection::Headers {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let headers_block = Block::default()
        .borders(Borders::ALL)
        .border_style(headers_style)
        .title(format!(" Headers ({}) ", app.current_request.headers.len()));
    let headers_inner = headers_block.inner(chunks[1]);
    frame.render_widget(headers_block, chunks[1]);

    if app.current_request.headers.is_empty() {
        let msg = Paragraph::new("No headers. Press 'h' to add.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, headers_inner);
    } else {
        let items: Vec<ListItem> = app
            .current_request
            .headers
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let selected = app.section == RequestSection::Headers && i == app.header_index;
                let bg = if selected { Color::DarkGray } else { Color::Reset };
                let enabled_style = if h.enabled {
                    Style::default().bg(bg)
                } else {
                    Style::default().fg(Color::DarkGray).bg(bg)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(&h.key, Style::default().fg(Color::Yellow).bg(bg)),
                    Span::styled(": ", enabled_style),
                    Span::styled(&h.value, enabled_style),
                ]))
            })
            .collect();
        let list = List::new(items);
        frame.render_widget(list, headers_inner);
    }

    // Body
    let body_style = if app.section == RequestSection::Body {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let body_block = Block::default()
        .borders(Borders::ALL)
        .border_style(body_style)
        .title(" Body ");
    let body_text = app
        .current_request
        .body
        .as_deref()
        .unwrap_or("No body. Press 'b' to edit.");
    let body = Paragraph::new(body_text)
        .block(body_block)
        .wrap(Wrap { trim: true });
    frame.render_widget(body, chunks[2]);
}

fn render_response(frame: &mut Frame, app: &App, area: Rect) {
    let Some(response) = &app.response else {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Response ");
        let msg = Paragraph::new("No response yet. Send a request first.")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Status
            Constraint::Min(1),    // Body
        ])
        .split(area);

    // Status line
    let status_color = if response.status < 300 {
        Color::Green
    } else if response.status < 400 {
        Color::Yellow
    } else {
        Color::Red
    };

    let status_text = format!(
        "{} {} | {}ms | {} bytes",
        response.status, response.status_text, response.duration_ms, response.size_bytes
    );
    let status_block = Block::default().borders(Borders::ALL).title(" Status ");
    let status = Paragraph::new(Span::styled(status_text, Style::default().fg(status_color)))
        .block(status_block);
    frame.render_widget(status, chunks[0]);

    // Body
    let body_block = Block::default().borders(Borders::ALL).title(" Body ");
    let body_inner = body_block.inner(chunks[1]);
    frame.render_widget(body_block, chunks[1]);

    let lines: Vec<Line> = response
        .body
        .lines()
        .skip(app.response_scroll)
        .map(|l| Line::from(l))
        .collect();
    let body = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(body, body_inner);
}

fn render_collections(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Saved Requests ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.requests.is_empty() {
        let msg = Paragraph::new("No saved requests. Press Ctrl+S in request view to save.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .requests
        .iter()
        .enumerate()
        .map(|(i, req)| {
            let selected = i == app.request_index;
            let bg = if selected { Color::DarkGray } else { Color::Reset };
            let method_color = match req.method {
                crate::request::Method::GET => Color::Green,
                crate::request::Method::POST => Color::Yellow,
                crate::request::Method::PUT => Color::Blue,
                crate::request::Method::DELETE => Color::Red,
                _ => Color::White,
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:7} ", req.method.as_str()),
                    Style::default().fg(method_color).bg(bg),
                ),
                Span::styled(&req.name, Style::default().bg(bg)),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_history(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" History ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.history.is_empty() {
        let msg = Paragraph::new("No history yet.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .history
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let selected = i == app.history_index;
            let bg = if selected { Color::DarkGray } else { Color::Reset };
            let status_color = if entry.status < 300 {
                Color::Green
            } else if entry.status < 400 {
                Color::Yellow
            } else {
                Color::Red
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:7} ", entry.method.as_str()),
                    Style::default().bg(bg),
                ),
                Span::styled(
                    format!("{} ", entry.status),
                    Style::default().fg(status_color).bg(bg),
                ),
                Span::styled(&entry.url, Style::default().bg(bg)),
                Span::styled(
                    format!(" ({}ms)", entry.duration_ms),
                    Style::default().fg(Color::DarkGray).bg(bg),
                ),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            "API Tester Help",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Request View",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Tab          Cycle sections"),
        Line::from("  m            Change method"),
        Line::from("  u            Edit URL"),
        Line::from("  h            Add header"),
        Line::from("  b            Edit body"),
        Line::from("  j/k          Navigate headers"),
        Line::from("  d            Delete header"),
        Line::from("  Ctrl+R, F5   Send request"),
        Line::from("  Ctrl+S       Save request"),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  c            Collections"),
        Line::from("  H            History"),
        Line::from("  v            View response"),
        Line::from("  n            New request"),
        Line::from(""),
        Line::from(Span::styled(
            "Response View",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  j/k          Scroll"),
        Line::from("  y            Copy as curl"),
        Line::from("  Esc          Back to request"),
        Line::from(""),
        Line::from(Span::styled(
            "General",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  ?            Toggle help"),
        Line::from("  q            Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .wrap(Wrap { trim: false });
    frame.render_widget(help, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let view_name = match app.view {
        View::Request => "Request",
        View::Response => "Response",
        View::Collections => "Collections",
        View::History => "History",
        View::Help => "Help",
    };

    let loading = if app.is_loading { " [Loading...]" } else { "" };

    let message = app
        .message
        .as_deref()
        .or(app.error.as_deref())
        .unwrap_or("? Help | Ctrl+R Send | q Quit");

    let style = if app.error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    let status = Paragraph::new(format!(" [{}]{} {} ", view_name, loading, message)).style(style);
    frame.render_widget(status, area);
}

fn render_input_dialog(frame: &mut Frame, title: &str, value: &str) {
    let area = centered_rect(60, 20, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input = Paragraph::new(format!("{}|", value));
    frame.render_widget(input, inner);
}

fn render_text_dialog(frame: &mut Frame, title: &str, value: &str) {
    let area = centered_rect(70, 50, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} (Esc to save) ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input = Paragraph::new(format!("{}|", value)).wrap(Wrap { trim: false });
    frame.render_widget(input, inner);
}

fn render_header_dialog(frame: &mut Frame, key: &str, value: &str) {
    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Add Header ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = format!("Key: {}\nValue: {}|", key, value);
    let input = Paragraph::new(text);
    frame.render_widget(input, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
