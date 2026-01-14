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
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_input(frame, app, chunks[1]);
    render_output(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let mode_color = match app.mode {
        Mode::Encode => Color::Green,
        Mode::Decode => Color::Blue,
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "BASE64 TOOL",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{:?}", app.mode),
            Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::raw(format!("Input: {} chars", app.input.len())),
        Span::raw(" → "),
        Span::raw(format!("Output: {} chars", app.output.len())),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Encoder/Decoder "));

    frame.render_widget(header, area);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let title = match app.mode {
        Mode::Encode => " Input (Plain Text) ",
        Mode::Decode => " Input (Base64) ",
    };

    let display_text = if app.input.is_empty() {
        "Type here...".to_string()
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
    let title = match app.mode {
        Mode::Encode => " Output (Base64) ",
        Mode::Decode => " Output (Plain Text) ",
    };

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

    let display_text = if app.output.is_empty() {
        "Output will appear here...".to_string()
    } else {
        app.output.clone()
    };

    let style = if app.output.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Green)
    };

    let output = Paragraph::new(display_text)
        .style(style)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });

    frame.render_widget(output, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
