use chrono::Local;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{App, Mode};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_current_time(frame, app, chunks[1]);
    render_input(frame, app, chunks[2]);
    render_output(frame, app, chunks[3]);
    render_status(frame, app, chunks[4]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let mode_str = match app.mode {
        Mode::EpochToDate => "Epoch → Date",
        Mode::DateToEpoch => "Date → Epoch",
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "EPOCH CONVERTER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(mode_str, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(
            app.unit.as_str(),
            Style::default().fg(Color::Green),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Timestamp Tool "));

    frame.render_widget(header, area);
}

fn render_current_time(frame: &mut Frame, app: &App, area: Rect) {
    let local = app.current_datetime.with_timezone(&Local);

    let content = vec![
        Line::from(vec![
            Span::styled("UTC:   ", Style::default().fg(Color::Cyan)),
            Span::raw(app.current_datetime.format("%Y-%m-%d %H:%M:%S").to_string()),
        ]),
        Line::from(vec![
            Span::styled("Local: ", Style::default().fg(Color::Cyan)),
            Span::raw(local.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        ]),
        Line::from(vec![
            Span::styled("Epoch: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                app.current_epoch.to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
    ];

    let current = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Current Time "));

    frame.render_widget(current, area);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let title = match app.mode {
        Mode::EpochToDate => " Input (Unix Timestamp) ",
        Mode::DateToEpoch => " Input (Date: YYYY-MM-DD HH:MM:SS) ",
    };

    let display_text = if app.input.is_empty() {
        match app.mode {
            Mode::EpochToDate => "Enter timestamp (e.g., 1609459200)...",
            Mode::DateToEpoch => "Enter date (e.g., 2021-01-01 00:00:00)...",
        }
        .to_string()
    } else {
        format!("{}_", app.input)
    };

    let style = if app.input.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(display_text)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(input, area);
}

fn render_output(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref error) = app.error {
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Error ")
                    .border_style(Style::default().fg(Color::Red)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(error_widget, area);
        return;
    }

    let content: Vec<Line> = match app.mode {
        Mode::EpochToDate => {
            if let Some(dt) = app.converted_datetime {
                let local = dt.with_timezone(&Local);
                vec![
                    Line::from(vec![
                        Span::styled("UTC:   ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                            Style::default().fg(Color::Green),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("Local: ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            local.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
                            Style::default().fg(Color::Green),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("ISO:   ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            dt.format("%+").to_string(),
                            Style::default().fg(Color::Green),
                        ),
                    ]),
                ]
            } else {
                vec![Line::styled(
                    "Converted date will appear here...",
                    Style::default().fg(Color::DarkGray),
                )]
            }
        }
        Mode::DateToEpoch => {
            if let Some(epoch) = app.converted_epoch {
                vec![
                    Line::from(vec![
                        Span::styled("Epoch (", Style::default().fg(Color::Cyan)),
                        Span::raw(app.unit.as_str()),
                        Span::styled("): ", Style::default().fg(Color::Cyan)),
                        Span::styled(epoch.to_string(), Style::default().fg(Color::Green)),
                    ]),
                ]
            } else {
                vec![Line::styled(
                    "Converted timestamp will appear here...",
                    Style::default().fg(Color::DarkGray),
                )]
            }
        }
    };

    let output = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Result "))
        .wrap(Wrap { trim: false });

    frame.render_widget(output, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
