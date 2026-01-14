use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph},
};

use crate::app::{App, SessionType, TimerState};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_timer(frame, app, chunks[1]);
    render_progress(frame, app, chunks[2]);
    render_stats(frame, app, chunks[3]);
    render_status(frame, app, chunks[4]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let state_str = match app.state {
        TimerState::Idle => "Ready",
        TimerState::Running => "Running",
        TimerState::Paused => "Paused",
        TimerState::Finished => "Finished",
    };

    let state_color = match app.state {
        TimerState::Idle => Color::Gray,
        TimerState::Running => Color::Green,
        TimerState::Paused => Color::Yellow,
        TimerState::Finished => Color::Cyan,
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "POMODORO TIMER",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(state_str, Style::default().fg(state_color)),
        Span::raw(" | "),
        Span::raw(format!("Work: {}min", app.work_duration.as_secs() / 60)),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Pomodoro "));

    frame.render_widget(header, area);
}

fn render_timer(frame: &mut Frame, app: &App, area: Rect) {
    let session_color = match app.session_type {
        SessionType::Work => Color::Red,
        SessionType::ShortBreak => Color::Green,
        SessionType::LongBreak => Color::Blue,
    };

    let time_str = App::format_duration(app.remaining);

    let timer_content = vec![
        Line::from(""),
        Line::from(Span::styled(
            app.session_type.label(),
            Style::default().fg(session_color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            time_str,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    let timer = Paragraph::new(timer_content)
        .block(Block::default().borders(Borders::ALL).title(" Timer "))
        .alignment(Alignment::Center);

    frame.render_widget(timer, area);
}

fn render_progress(frame: &mut Frame, app: &App, area: Rect) {
    let color = match app.session_type {
        SessionType::Work => Color::Red,
        SessionType::ShortBreak => Color::Green,
        SessionType::LongBreak => Color::Blue,
    };

    let progress = app.progress();
    let percentage = (progress * 100.0) as u16;

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Progress "))
        .gauge_style(Style::default().fg(color))
        .ratio(progress)
        .label(format!("{}%", percentage));

    frame.render_widget(gauge, area);
}

fn render_stats(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_session_stats(frame, app, chunks[0]);
    render_cycle_indicator(frame, app, chunks[1]);
}

fn render_session_stats(frame: &mut Frame, app: &App, area: Rect) {
    let total_hours = app.total_work_time.as_secs() / 3600;
    let total_mins = (app.total_work_time.as_secs() % 3600) / 60;

    let stats = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Completed: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} pomodoros", app.completed_pomodoros),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled("Current Streak: ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.current_streak.to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("Total Work: ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{}h {}m", total_hours, total_mins)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Statistics "));

    frame.render_widget(stats, area);
}

fn render_cycle_indicator(frame: &mut Frame, app: &App, area: Rect) {
    let total = app.sessions_until_long_break;
    let current = app.sessions_in_cycle;

    let indicators: String = (0..total)
        .map(|i| {
            if i < current {
                "● "
            } else {
                "○ "
            }
        })
        .collect();

    let next_session = if current >= total {
        "Long Break"
    } else if current == total - 1 {
        "1 more until Long Break"
    } else {
        "Short Break after Work"
    };

    let cycle = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Cycle: ", Style::default().fg(Color::Gray)),
            Span::styled(indicators, Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::styled("Next: ", Style::default().fg(Color::Gray)),
            Span::raw(next_session),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "4 pomodoros = 1 long break",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Cycle "));

    frame.render_widget(cycle, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
