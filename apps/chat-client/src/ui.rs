use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::app::{App, InputMode};
use crate::models::{MessageType, UserStatus};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Title bar
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Input
            Constraint::Length(1),  // Status
        ])
        .split(area);

    render_title_bar(frame, app, chunks[0]);
    render_main_content(frame, app, chunks[1]);
    render_input(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);

    // Render overlays
    match app.input_mode {
        InputMode::ServerList => render_server_list(frame, app),
        InputMode::ChannelList => render_channel_list(frame, app),
        _ => {}
    }
}

fn render_title_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (server_name, channel_info) = if let (Some(server), Some(channel)) =
        (app.current_server(), app.current_channel())
    {
        let topic = channel
            .topic
            .as_deref()
            .map(|t| format!(" - {}", t))
            .unwrap_or_default();
        (
            server.display_name(),
            format!("{}{}", channel.display_name(), topic),
        )
    } else {
        ("No server".to_string(), String::new())
    };

    let title = format!(" {} | {} ", server_name, channel_info);
    let paragraph = Paragraph::new(title)
        .style(Style::default().bg(Color::Blue).fg(Color::White));

    frame.render_widget(paragraph, area);
}

fn render_main_content(frame: &mut Frame, app: &App, area: Rect) {
    if app.show_user_list {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(30), Constraint::Length(20)])
            .split(area);

        render_messages(frame, app, chunks[0]);
        render_user_list(frame, app, chunks[1]);
    } else {
        render_messages(frame, app, area);
    }
}

fn render_messages(frame: &mut Frame, app: &App, area: Rect) {
    let Some(channel) = app.current_channel() else {
        let empty = Paragraph::new("No channel selected")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, area);
        return;
    };

    let messages: Vec<Line> = channel
        .messages
        .iter()
        .map(|msg| {
            let timestamp = if app.config.display.show_timestamps {
                format!("[{}] ", msg.timestamp.format(&app.config.display.timestamp_format))
            } else {
                String::new()
            };

            match msg.message_type {
                MessageType::Chat => {
                    let sender_style = if msg.is_mention {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };

                    Line::from(vec![
                        Span::styled(timestamp, Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("<{}> ", msg.sender), sender_style),
                        Span::raw(&msg.content),
                    ])
                }
                MessageType::System => Line::from(vec![
                    Span::styled(timestamp, Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("* {}", msg.content), Style::default().fg(Color::Green)),
                ]),
                MessageType::Action => Line::from(vec![
                    Span::styled(timestamp, Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("* {} {}", msg.sender, msg.content),
                        Style::default().fg(Color::Magenta),
                    ),
                ]),
                MessageType::Join => Line::from(vec![
                    Span::styled(timestamp, Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("--> {} has joined", msg.sender),
                        Style::default().fg(Color::Green),
                    ),
                ]),
                MessageType::Part | MessageType::Quit => Line::from(vec![
                    Span::styled(timestamp, Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("<-- {} has left: {}", msg.sender, msg.content),
                        Style::default().fg(Color::Red),
                    ),
                ]),
                MessageType::Notice => Line::from(vec![
                    Span::styled(timestamp, Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("-{}- {}", msg.sender, msg.content),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
            }
        })
        .collect();

    let paragraph = Paragraph::new(messages)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " {} ({} users) ",
            channel.display_name(),
            channel.users.len()
        )))
        .scroll((app.message_scroll as u16, 0));

    frame.render_widget(paragraph, area);

    // Scrollbar
    if channel.messages.len() > area.height as usize - 2 {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut scrollbar_state = ScrollbarState::new(channel.messages.len())
            .position(app.message_scroll);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin::new(0, 1)),
            &mut scrollbar_state,
        );
    }
}

fn render_user_list(frame: &mut Frame, app: &App, area: Rect) {
    let Some(channel) = app.current_channel() else {
        return;
    };

    let users: Vec<ListItem> = channel
        .users
        .iter()
        .map(|user| {
            let status_color = match user.status {
                UserStatus::Online => Color::Green,
                UserStatus::Away => Color::Yellow,
                UserStatus::Busy => Color::Red,
                UserStatus::Offline => Color::DarkGray,
            };

            ListItem::new(Line::from(vec![
                Span::styled("● ", Style::default().fg(status_color)),
                Span::raw(user.nick_prefix()),
                Span::raw(user.name()),
            ]))
        })
        .collect();

    let list = List::new(users)
        .block(Block::default().borders(Borders::ALL).title(" Users "));

    frame.render_widget(list, area);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let (title, content, style) = match app.input_mode {
        InputMode::Insert => (
            " Input ",
            format!("{}_", app.input_buffer),
            Style::default().fg(Color::Yellow),
        ),
        InputMode::Command => (
            " Command ",
            format!(":{}_", app.command_buffer),
            Style::default().fg(Color::Cyan),
        ),
        _ => (
            " Input ",
            app.input_buffer.clone(),
            Style::default(),
        ),
    };

    let paragraph = Paragraph::new(content)
        .style(style)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(paragraph, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let mode_style = match app.input_mode {
        InputMode::Normal => Style::default().bg(Color::DarkGray),
        InputMode::Insert => Style::default().bg(Color::Green).fg(Color::Black),
        InputMode::Command => Style::default().bg(Color::Blue).fg(Color::White),
        _ => Style::default().bg(Color::DarkGray),
    };

    let paragraph = Paragraph::new(format!(" {} ", status)).style(mode_style);

    frame.render_widget(paragraph, area);
}

fn render_server_list(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 50, frame.area());
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .servers
        .iter()
        .enumerate()
        .map(|(i, server)| {
            let style = if i == app.server_list_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let status = if server.connected {
                Span::styled("●", Style::default().fg(Color::Green))
            } else {
                Span::styled("○", Style::default().fg(Color::Red))
            };

            ListItem::new(Line::from(vec![
                status,
                Span::raw(" "),
                Span::raw(server.display_name()),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Servers "));

    frame.render_widget(list, area);
}

fn render_channel_list(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 50, frame.area());
    frame.render_widget(Clear, area);

    let channels: Vec<ListItem> = app
        .current_server()
        .map(|s| &s.channels)
        .unwrap_or(&vec![])
        .iter()
        .enumerate()
        .map(|(i, channel)| {
            let style = if i == app.channel_list_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let unread = if channel.unread_count > 0 {
                Span::styled(
                    format!(" ({})", channel.unread_count),
                    Style::default().fg(Color::Yellow),
                )
            } else {
                Span::raw("")
            };

            ListItem::new(Line::from(vec![
                Span::raw(channel.display_name()),
                unread,
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(channels)
        .block(Block::default().borders(Borders::ALL).title(" Channels "));

    frame.render_widget(list, area);
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
