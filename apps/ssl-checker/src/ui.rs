use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, FilterBy, InputMode, SortBy, View};
use crate::models::HostStatus;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(frame, app, chunks[0]);
    render_stats(frame, app, chunks[1]);
    render_main(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let stats = app.stats();

    let sort_indicator = match app.sort_by {
        SortBy::Name => "Name",
        SortBy::ExpiryDate => "Expiry",
        SortBy::Status => "Status",
    };

    let filter_indicator = match app.filter {
        FilterBy::All => "All",
        FilterBy::Valid => "Valid",
        FilterBy::Warning => "Warning",
        FilterBy::Critical => "Critical",
        FilterBy::Expired => "Expired",
        FilterBy::Error => "Error",
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "SSL CHECKER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::raw(format!("{} hosts", stats.total)),
        Span::raw(" | Sort: "),
        Span::styled(sort_indicator, Style::default().fg(Color::Yellow)),
        Span::raw(" | Filter: "),
        Span::styled(filter_indicator, Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Certificate Monitor "));

    frame.render_widget(header, area);
}

fn render_stats(frame: &mut Frame, app: &App, area: Rect) {
    let stats = app.stats();

    let line = Line::from(vec![
        Span::styled("✓ ", Style::default().fg(Color::Green)),
        Span::raw(format!("{} ", stats.valid)),
        Span::styled("⚠ ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{} ", stats.warning)),
        Span::styled("! ", Style::default().fg(Color::LightRed)),
        Span::raw(format!("{} ", stats.critical)),
        Span::styled("✗ ", Style::default().fg(Color::Red)),
        Span::raw(format!("{} ", stats.expired)),
        Span::styled("⚡ ", Style::default().fg(Color::Magenta)),
        Span::raw(format!("{}", stats.error)),
    ]);

    let stats_bar = Paragraph::new(line)
        .block(Block::default().borders(Borders::ALL).title(" Status Summary "))
        .alignment(Alignment::Center);

    frame.render_widget(stats_bar, area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::List => render_list(frame, app, area),
        View::Details => render_details(frame, app, area),
        View::Add => render_add(frame, app, area),
    }
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_hosts();

    if filtered.is_empty() {
        let empty = Paragraph::new("No hosts match the current filter. Press 'a' to add one.")
            .block(Block::default().borders(Borders::ALL).title(" Hosts "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, host)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let status = host.status();
            let status_style = match status {
                HostStatus::Valid => Style::default().fg(Color::Green),
                HostStatus::Warning => Style::default().fg(Color::Yellow),
                HostStatus::Critical => Style::default().fg(Color::LightRed),
                HostStatus::Expired => Style::default().fg(Color::Red),
                HostStatus::Error => Style::default().fg(Color::Magenta),
                HostStatus::Unknown => Style::default().fg(Color::DarkGray),
            };

            let days = host.days_until_expiry()
                .map(|d| {
                    if d < 0 {
                        format!("{} days ago", -d)
                    } else {
                        format!("{} days", d)
                    }
                })
                .unwrap_or_else(|| "N/A".to_string());

            let issuer = host.certificate.as_ref()
                .map(|c| c.issuer.replace("CN=", ""))
                .unwrap_or_else(|| "-".to_string());

            let error_or_issuer = if let Some(ref err) = host.error {
                err.clone()
            } else {
                issuer
            };

            Row::new(vec![
                Cell::from(Span::styled(status.icon(), status_style)),
                Cell::from(format!("{}:{}", host.hostname, host.port)),
                Cell::from(Span::styled(status.label(), status_style)),
                Cell::from(days),
                Cell::from(error_or_issuer),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Min(25),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec!["", "Host", "Status", "Expires In", "Issuer / Error"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(Block::default().borders(Borders::ALL).title(format!(
        " Certificates ({} shown) ",
        filtered.len()
    )));

    frame.render_widget(table, area);
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    let Some(host) = app.selected_host() else {
        let empty = Paragraph::new("No host selected")
            .block(Block::default().borders(Borders::ALL).title(" Certificate Details "));
        frame.render_widget(empty, area);
        return;
    };

    let status = host.status();
    let status_style = match status {
        HostStatus::Valid => Style::default().fg(Color::Green),
        HostStatus::Warning => Style::default().fg(Color::Yellow),
        HostStatus::Critical => Style::default().fg(Color::LightRed),
        HostStatus::Expired => Style::default().fg(Color::Red),
        HostStatus::Error => Style::default().fg(Color::Magenta),
        HostStatus::Unknown => Style::default().fg(Color::DarkGray),
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Host: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}:{}", host.hostname, host.port),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{} {}", status.icon(), status.label()), status_style),
        ]),
        Line::from(""),
    ];

    if let Some(ref err) = host.error {
        lines.push(Line::from(vec![
            Span::styled("Error: ", Style::default().fg(Color::Red)),
            Span::raw(err),
        ]));
    } else if let Some(ref cert) = host.certificate {
        lines.extend(vec![
            Line::from(Span::styled(
                "Certificate Information:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Subject: ", Style::default().fg(Color::Gray)),
                Span::raw(&cert.subject),
            ]),
            Line::from(vec![
                Span::styled("Issuer: ", Style::default().fg(Color::Gray)),
                Span::raw(&cert.issuer),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Valid From: ", Style::default().fg(Color::Gray)),
                Span::raw(cert.not_before.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
            ]),
            Line::from(vec![
                Span::styled("Valid Until: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    cert.not_after.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                    if cert.is_valid() {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Days Remaining: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", host.days_until_expiry().unwrap_or(0)),
                    status_style,
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Serial: ", Style::default().fg(Color::Gray)),
                Span::raw(&cert.serial_number),
            ]),
            Line::from(vec![
                Span::styled("Algorithm: ", Style::default().fg(Color::Gray)),
                Span::raw(&cert.signature_algorithm),
            ]),
            Line::from(vec![
                Span::styled("Self-Signed: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    if cert.is_self_signed { "Yes" } else { "No" },
                    if cert.is_self_signed {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Subject Alt Names: ", Style::default().fg(Color::Gray)),
                Span::raw(cert.san.join(", ")),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("SHA256 Fingerprint: ", Style::default().fg(Color::Gray)),
                Span::styled(&cert.fingerprint_sha256, Style::default().fg(Color::DarkGray)),
            ]),
        ]);
    }

    if let Some(checked) = host.last_checked {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Last Checked: ", Style::default().fg(Color::Gray)),
            Span::raw(checked.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Certificate Details "))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_add(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(5),
        ])
        .split(area);

    // Hostname field
    let hostname_style = if app.input_mode == InputMode::EditHostname {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let hostname_content = if app.input_mode == InputMode::EditHostname {
        format!("{}_", app.hostname_buffer)
    } else {
        app.hostname_buffer.clone()
    };

    let hostname_block = Block::default()
        .borders(Borders::ALL)
        .title(" Hostname ")
        .border_style(hostname_style);

    let hostname_para = Paragraph::new(hostname_content).block(hostname_block);
    frame.render_widget(hostname_para, chunks[0]);

    // Port field
    let port_style = if app.input_mode == InputMode::EditPort {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let port_content = if app.input_mode == InputMode::EditPort {
        format!("{}_", app.port_buffer)
    } else {
        app.port_buffer.clone()
    };

    let port_block = Block::default()
        .borders(Borders::ALL)
        .title(" Port ")
        .border_style(port_style);

    let port_para = Paragraph::new(port_content).block(port_block);
    frame.render_widget(port_para, chunks[1]);

    // Help text
    let help = Paragraph::new(vec![
        Line::from(""),
        Line::from("Enter the hostname and port of the SSL endpoint to monitor."),
        Line::from(""),
        Line::from("Examples:"),
        Line::from(Span::styled("  example.com:443", Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled("  api.service.com:8443", Style::default().fg(Color::DarkGray))),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Help "));

    frame.render_widget(help, chunks[2]);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let style = Style::default().bg(Color::DarkGray);
    let paragraph = Paragraph::new(format!(" {} ", status)).style(style);
    frame.render_widget(paragraph, area);
}
