use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{App, TestState};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(6),
            Constraint::Length(6),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_stats(frame, app, chunks[1]);
    render_text(frame, app, chunks[2]);
    render_results(frame, app, chunks[3]);
    render_status(frame, app, chunks[4]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let state_str = match app.state {
        TestState::Ready => "Ready",
        TestState::Running => "Running",
        TestState::Finished => "Finished",
    };

    let state_color = match app.state {
        TestState::Ready => Color::Yellow,
        TestState::Running => Color::Green,
        TestState::Finished => Color::Cyan,
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "TYPING TEST",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(state_str, Style::default().fg(state_color)),
        Span::raw(" | "),
        Span::styled(
            format!("{} tests completed", app.results.len()),
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Speed Test "));

    frame.render_widget(header, area);
}

fn render_stats(frame: &mut Frame, app: &App, area: Rect) {
    let wpm = if app.state == TestState::Running {
        app.live_wpm()
    } else if let Some(ref result) = app.current_result {
        result.wpm
    } else {
        0.0
    };

    let accuracy = if app.state == TestState::Running {
        app.live_accuracy()
    } else if let Some(ref result) = app.current_result {
        result.accuracy
    } else {
        100.0
    };

    let elapsed = app.elapsed_seconds();

    let stats_text = vec![
        Line::from(vec![
            Span::styled("WPM: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.0}", wpm),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  |  "),
            Span::styled("Accuracy: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1}%", accuracy),
                Style::default()
                    .fg(if accuracy >= 95.0 {
                        Color::Green
                    } else if accuracy >= 85.0 {
                        Color::Yellow
                    } else {
                        Color::Red
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  |  "),
            Span::styled("Time: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1}s", elapsed),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("Average WPM: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.0}", app.average_wpm()),
                Style::default().fg(Color::Magenta),
            ),
            Span::raw("  |  "),
            Span::styled("Progress: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}/{}", app.typed_text.len(), app.target_text.len()),
                Style::default().fg(Color::Yellow),
            ),
        ]),
    ];

    let stats = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title(" Statistics "));

    frame.render_widget(stats, area);
}

fn render_text(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();

    for (i, target_char) in app.target_text.chars().enumerate() {
        if i < app.typed_text.len() {
            let typed_char = app.typed_text.chars().nth(i);
            if app.errors.contains(&i) {
                // Error - show in red
                spans.push(Span::styled(
                    target_char.to_string(),
                    Style::default().fg(Color::Red).bg(Color::DarkGray),
                ));
            } else {
                // Correct - show in green
                spans.push(Span::styled(
                    target_char.to_string(),
                    Style::default().fg(Color::Green),
                ));
            }
        } else if i == app.typed_text.len() {
            // Current position - highlight
            spans.push(Span::styled(
                target_char.to_string(),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            // Not yet typed
            spans.push(Span::styled(
                target_char.to_string(),
                Style::default().fg(Color::Gray),
            ));
        }
    }

    let text = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL).title(" Text "))
        .wrap(Wrap { trim: false });

    frame.render_widget(text, area);
}

fn render_results(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref result) = app.current_result {
        let result_text = vec![
            Line::from(vec![
                Span::styled("Final WPM: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.1}", result.wpm),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Accuracy: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.1}%", result.accuracy),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("({} correct, {} errors)", result.correct, result.incorrect),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(vec![
                Span::styled("Time: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.2} seconds", result.time_seconds),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        let results = Paragraph::new(result_text)
            .block(Block::default().borders(Borders::ALL).title(" Results "));

        frame.render_widget(results, area);
    } else {
        let placeholder = Paragraph::new("Complete a test to see results")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        frame.render_widget(placeholder, area);
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
