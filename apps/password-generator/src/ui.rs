use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

use crate::app::{App, PasswordStrength, PasswordType};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_password(frame, app, chunks[1]);
    render_strength(frame, app, chunks[2]);
    render_options(frame, app, chunks[3]);
    render_status(frame, app, chunks[4]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let type_name = match app.password_type {
        PasswordType::Random => "Random",
        PasswordType::Memorable => "Memorable",
        PasswordType::Pin => "PIN",
        PasswordType::Passphrase => "Passphrase",
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "PASSWORD GENERATOR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(type_name, Style::default().fg(Color::Yellow)),
        Span::raw(" | Length: "),
        Span::styled(
            app.length.to_string(),
            Style::default().fg(Color::Green),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Generator "));

    frame.render_widget(header, area);
}

fn render_password(frame: &mut Frame, app: &App, area: Rect) {
    let password = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            &app.password,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Generated Password "))
    .alignment(Alignment::Center);

    frame.render_widget(password, area);
}

fn render_strength(frame: &mut Frame, app: &App, area: Rect) {
    let (color, ratio) = match app.strength {
        PasswordStrength::VeryWeak => (Color::Red, 0.2),
        PasswordStrength::Weak => (Color::LightRed, 0.4),
        PasswordStrength::Medium => (Color::Yellow, 0.6),
        PasswordStrength::Strong => (Color::LightGreen, 0.8),
        PasswordStrength::VeryStrong => (Color::Green, 1.0),
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Strength "))
        .gauge_style(Style::default().fg(color))
        .ratio(ratio)
        .label(app.strength.label());

    frame.render_widget(gauge, area);
}

fn render_options(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_settings(frame, app, chunks[0]);
    render_history(frame, app, chunks[1]);
}

fn render_settings(frame: &mut Frame, app: &App, area: Rect) {
    let checkbox = |enabled: bool| if enabled { "[x]" } else { "[ ]" };

    let items = vec![
        ListItem::new(Line::from(vec![
            Span::raw(format!("{} ", checkbox(app.include_uppercase))),
            Span::styled("u", Style::default().fg(Color::Yellow)),
            Span::raw("ppercase (A-Z)"),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw(format!("{} ", checkbox(app.include_lowercase))),
            Span::styled("l", Style::default().fg(Color::Yellow)),
            Span::raw("owercase (a-z)"),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw(format!("{} ", checkbox(app.include_digits))),
            Span::styled("d", Style::default().fg(Color::Yellow)),
            Span::raw("igits (0-9)"),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw(format!("{} ", checkbox(app.include_symbols))),
            Span::styled("s", Style::default().fg(Color::Yellow)),
            Span::raw("ymbols (!@#$...)"),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw(format!("{} ", checkbox(app.exclude_ambiguous))),
            Span::raw("Exclude "),
            Span::styled("a", Style::default().fg(Color::Yellow)),
            Span::raw("mbiguous (l1I0O)"),
        ])),
    ];

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Options "));

    frame.render_widget(list, area);
}

fn render_history(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .history
        .iter()
        .rev()
        .take(8)
        .enumerate()
        .map(|(i, pwd)| {
            let style = if i == 0 {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(Span::styled(truncate(pwd, 30), style))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" History "));

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
