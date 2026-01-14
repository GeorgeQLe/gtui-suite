use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, Security, View};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);

    match app.view {
        View::Networks => render_networks(frame, app, chunks[1]),
        View::Details => render_details(frame, app, chunks[1]),
        View::Connect => render_connect(frame, app, chunks[1]),
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let wifi_status = if app.wifi_enabled {
        Span::styled("ON", Style::default().fg(Color::Green))
    } else {
        Span::styled("OFF", Style::default().fg(Color::Red))
    };

    let connected_info = app
        .connected_network()
        .map(|n| format!(" | Connected: {}", n.ssid))
        .unwrap_or_else(|| " | Not connected".to_string());

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "WIFI MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | WiFi: "),
        wifi_status,
        Span::raw(&connected_info),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Network Manager "));

    frame.render_widget(header, area);
}

fn render_networks(frame: &mut Frame, app: &App, area: Rect) {
    if app.networks.is_empty() {
        let empty = Paragraph::new("No networks found. Press 'r' to scan.")
            .block(Block::default().borders(Borders::ALL).title(" Networks "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .networks
        .iter()
        .enumerate()
        .map(|(i, network)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let connected_icon = if network.connected { "●" } else { " " };
            let connected_style = if network.connected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            let saved_icon = if network.saved { "★" } else { " " };

            let signal_style = match network.signal_bars() {
                4 => Style::default().fg(Color::Green),
                3 => Style::default().fg(Color::LightGreen),
                2 => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::Red),
            };

            let line = Line::from(vec![
                Span::styled(connected_icon, connected_style),
                Span::raw(" "),
                Span::styled(network.signal_icon(), signal_style),
                Span::raw(" "),
                Span::raw(format!("{} ", network.security.icon())),
                Span::styled(&network.ssid, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(saved_icon, Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled(
                    format!("{}dBm", network.signal_strength),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(" "),
                Span::styled(
                    &network.frequency,
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Networks ({}) ", app.networks.len())),
    );

    frame.render_widget(list, area);
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    let Some(network) = app.selected_network() else {
        return;
    };

    let signal_style = match network.signal_bars() {
        4 => Style::default().fg(Color::Green),
        3 => Style::default().fg(Color::LightGreen),
        2 => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::Red),
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("SSID: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &network.ssid,
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            if network.connected {
                Span::styled("Connected", Style::default().fg(Color::Green))
            } else {
                Span::styled("Not connected", Style::default().fg(Color::DarkGray))
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Security: ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{} {}", network.security.icon(), network.security.as_str())),
        ]),
        Line::from(vec![
            Span::styled("Signal: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} ({}dBm)", network.signal_icon(), network.signal_strength),
                signal_style,
            ),
        ]),
        Line::from(vec![
            Span::styled("Frequency: ", Style::default().fg(Color::Gray)),
            Span::raw(&network.frequency),
        ]),
        Line::from(vec![
            Span::styled("BSSID: ", Style::default().fg(Color::Gray)),
            Span::styled(&network.bssid, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Saved: ", Style::default().fg(Color::Gray)),
            if network.saved {
                Span::styled("Yes ★", Style::default().fg(Color::Yellow))
            } else {
                Span::raw("No")
            },
        ]),
    ];

    let details = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Network Details "));

    frame.render_widget(details, area);
}

fn render_connect(frame: &mut Frame, app: &App, area: Rect) {
    let Some(network) = app.selected_network() else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(5),
        ])
        .split(area);

    // Network name
    let name = Paragraph::new(Line::from(vec![
        Span::raw("Connecting to: "),
        Span::styled(
            &network.ssid,
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" ({})", network.security.as_str())),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Connect "));

    frame.render_widget(name, chunks[0]);

    // Password input
    let password_display = if app.show_password {
        format!("{}_", app.password_input)
    } else {
        format!("{}_", "*".repeat(app.password_input.len()))
    };

    let password = Paragraph::new(vec![
        Line::from(""),
        Line::from(password_display),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Password (Tab to show/hide) ")
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(password, chunks[1]);

    // Help
    let help = Paragraph::new(vec![
        Line::from(""),
        Line::from("Enter the password and press Enter to connect."),
        Line::from("Press Esc to cancel."),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Help "));

    frame.render_widget(help, chunks[2]);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
