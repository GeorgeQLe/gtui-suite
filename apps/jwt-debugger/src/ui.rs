use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_input(frame, app, chunks[1]);
    render_decoded(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let status = if let Some(ref decoded) = app.decoded {
        match decoded.expired {
            Some(true) => ("EXPIRED", Color::Red),
            Some(false) => ("VALID", Color::Green),
            None => ("NO EXP", Color::Yellow),
        }
    } else if app.error.is_some() {
        ("ERROR", Color::Red)
    } else {
        ("READY", Color::Gray)
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "JWT DEBUGGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(status.0, Style::default().fg(status.1)),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Token Decoder "));

    frame.render_widget(header, area);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let display = if app.input.is_empty() {
        "Paste or type JWT token here...".to_string()
    } else {
        format!("{}_", app.input)
    };

    let style = if app.input.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(display)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" JWT Token ")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(input, area);
}

fn render_decoded(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref error) = app.error {
        let err = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  Error: {}", error),
                Style::default().fg(Color::Red),
            )),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Decoded "));
        frame.render_widget(err, area);
        return;
    }

    let Some(ref decoded) = app.decoded else {
        let placeholder = Paragraph::new("Decoded JWT will appear here")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Decoded "));
        frame.render_widget(placeholder, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(area);

    // Header
    let header_text = if let Some(ref json) = decoded.header_json {
        format_json(json)
    } else {
        vec![Line::from(Span::styled(
            "  Failed to decode header",
            Style::default().fg(Color::Red),
        ))]
    };

    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Header ")
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(header, chunks[0]);

    // Payload
    let mut payload_text = if let Some(ref json) = decoded.payload_json {
        format_json(json)
    } else {
        vec![Line::from(Span::styled(
            "  Failed to decode payload",
            Style::default().fg(Color::Red),
        ))]
    };

    if let Some(ref exp_time) = decoded.exp_time {
        payload_text.push(Line::from(""));
        payload_text.push(Line::from(vec![
            Span::styled("  Expires: ", Style::default().fg(Color::Gray)),
            Span::styled(
                exp_time,
                Style::default().fg(if decoded.expired == Some(true) {
                    Color::Red
                } else {
                    Color::Green
                }),
            ),
        ]));
    }

    let payload = Paragraph::new(payload_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Payload ")
            .border_style(Style::default().fg(Color::Green)),
    );
    frame.render_widget(payload, chunks[1]);

    // Signature
    let sig_status = if decoded.signature.len() > 20 {
        format!("{}...", &decoded.signature[..20])
    } else {
        decoded.signature.clone()
    };

    let signature = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(sig_status, Style::default().fg(Color::Magenta)),
        Span::styled(" (not verified)", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Signature ")
            .border_style(Style::default().fg(Color::Magenta)),
    );
    frame.render_widget(signature, chunks[2]);
}

fn format_json(json: &serde_json::Value) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if let Some(obj) = json.as_object() {
        for (key, value) in obj {
            let value_str = match value {
                serde_json::Value::String(s) => format!("\"{}\"", s),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => value.to_string(),
            };

            lines.push(Line::from(vec![
                Span::styled(format!("  {}: ", key), Style::default().fg(Color::Cyan)),
                Span::styled(value_str, Style::default().fg(Color::White)),
            ]));
        }
    }

    lines
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
