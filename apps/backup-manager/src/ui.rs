use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table},
};

use crate::app::{App, ConfirmAction, Mode, ProfileFormState, View};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),     // Main content
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    render_content(frame, app, chunks[0]);
    render_status_bar(frame, app, chunks[1]);

    // Render overlays
    match &app.mode {
        Mode::AddProfile(form) => render_profile_form(frame, "Add Profile", form),
        Mode::EditProfile(form) => render_profile_form(frame, "Edit Profile", form),
        Mode::Confirm(action) => render_confirm_dialog(frame, action),
        Mode::Normal => {}
    }
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Dashboard => render_dashboard(frame, app, area),
        View::ProfileDetail => render_profile_detail(frame, app, area),
        View::Runs => render_runs(frame, app, area),
        View::Help => render_help(frame, area),
    }
}

fn render_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["", "Name", "Backend", "Destination", "Schedule", "Last Run"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app.profiles.iter().enumerate().map(|(i, profile)| {
        let style = if i == app.selected {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else if !profile.enabled {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let status = if profile.enabled { "●" } else { "○" };
        let last_run = app.get_last_run(&profile.id)
            .map(|r| format!("{} {}", r.status.icon(), r.started_at.format("%Y-%m-%d %H:%M")))
            .unwrap_or_else(|| "Never".to_string());

        Row::new(vec![
            status.to_string(),
            profile.name.clone(),
            profile.backend.label().to_string(),
            profile.destination.clone(),
            profile.schedule.clone().unwrap_or_else(|| "Manual".to_string()),
            last_run,
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(2),
        Constraint::Percentage(20),
        Constraint::Percentage(10),
        Constraint::Percentage(30),
        Constraint::Percentage(15),
        Constraint::Percentage(20),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Backup Profiles "));

    frame.render_widget(table, area);
}

fn render_profile_detail(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(profile) = app.selected_profile() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(12),
                Constraint::Min(1),
            ])
            .split(area);

        // Profile info
        let info = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&profile.name),
            ]),
            Line::from(vec![
                Span::styled("Backend: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(profile.backend.label()),
            ]),
            Line::from(vec![
                Span::styled("Sources: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(profile.sources_display()),
            ]),
            Line::from(vec![
                Span::styled("Destination: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&profile.destination),
            ]),
            Line::from(vec![
                Span::styled("Schedule: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(profile.schedule.as_deref().unwrap_or("Manual")),
            ]),
            Line::from(vec![
                Span::styled("Retention: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(profile.retention.display()),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(if profile.enabled { "Enabled" } else { "Disabled" }),
            ]),
        ];

        let info_block = Paragraph::new(info)
            .block(Block::default().borders(Borders::ALL).title(" Profile Details "));
        frame.render_widget(info_block, chunks[0]);

        // Recent runs
        let items: Vec<ListItem> = app.selected_runs.iter().map(|run| {
            let line = format!(
                "{} {} - {} ({})",
                run.status.icon(),
                run.started_at.format("%Y-%m-%d %H:%M"),
                run.status.label(),
                run.duration_display()
            );
            ListItem::new(line)
        }).collect();

        let runs = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Recent Runs "));
        frame.render_widget(runs, chunks[1]);
    }
}

fn render_runs(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Status", "Started", "Duration", "Files", "Size", "Error"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app.selected_runs.iter().enumerate().map(|(i, run)| {
        let style = if i == app.run_selected {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };

        Row::new(vec![
            format!("{} {}", run.status.icon(), run.status.label()),
            run.started_at.format("%Y-%m-%d %H:%M").to_string(),
            run.duration_display(),
            run.files_transferred.map(|f| f.to_string()).unwrap_or_default(),
            run.bytes_transferred.map(|b| crate::profile::format_bytes(b)).unwrap_or_default(),
            run.error_message.clone().unwrap_or_default(),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(15),
        Constraint::Percentage(20),
        Constraint::Percentage(12),
        Constraint::Percentage(10),
        Constraint::Percentage(13),
        Constraint::Percentage(30),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Backup Runs "));

    frame.render_widget(table, area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled("Backup Manager Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k, ↑/↓     Move up/down"),
        Line::from("  g/G          Top/bottom"),
        Line::from("  Enter        View details"),
        Line::from("  Esc          Back to dashboard"),
        Line::from(""),
        Line::from(Span::styled("Profile Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  a            Add profile"),
        Line::from("  e            Edit profile"),
        Line::from("  d            Delete profile"),
        Line::from("  t            Toggle enabled"),
        Line::from(""),
        Line::from(Span::styled("Backup Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  b            Run backup now"),
        Line::from("  l            View run logs"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  F5           Refresh"),
        Line::from("  ?            Show this help"),
        Line::from("  q            Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "));

    frame.render_widget(help, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let message = app.message.as_deref()
        .or(app.error.as_deref())
        .unwrap_or("? Help | a Add | b Backup | Enter Details | q Quit");

    let style = if app.error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    let count = format!(" {} profiles ", app.profiles.len());
    let status = Paragraph::new(format!("{} | {}", count, message)).style(style);
    frame.render_widget(status, area);
}

fn render_profile_form(frame: &mut Frame, title: &str, form: &ProfileFormState) {
    let area = centered_rect(60, 60, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            (0..ProfileFormState::field_count())
                .map(|_| Constraint::Length(2))
                .collect::<Vec<_>>()
        )
        .split(inner);

    for i in 0..ProfileFormState::field_count() {
        let label = ProfileFormState::field_label(i);
        let value = form.field_value(i);
        let is_active = i == form.field;

        let style = if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let display = if is_active {
            format!("{}: {}█", label, value)
        } else {
            format!("{}: {}", label, value)
        };

        let line = Paragraph::new(display).style(style);
        frame.render_widget(line, chunks[i]);
    }
}

fn render_confirm_dialog(frame: &mut Frame, action: &ConfirmAction) {
    let area = centered_rect(50, 25, frame.area());
    frame.render_widget(Clear, area);

    let message = match action {
        ConfirmAction::DeleteProfile(id) => format!("Delete profile {}?", &id[..8.min(id.len())]),
        ConfirmAction::RunBackup(id) => format!("Run backup for {}?", &id[..8.min(id.len())]),
    };

    let text = vec![
        Line::from(""),
        Line::from(message),
        Line::from(""),
        Line::from(Span::styled("(y)es / (n)o", Style::default().fg(Color::Yellow))),
    ];

    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
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
