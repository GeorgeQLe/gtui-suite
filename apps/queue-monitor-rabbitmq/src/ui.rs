use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs},
};

use crate::app::{App, ConfirmAction, InputMode, View};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    render_tabs(frame, app, chunks[0]);
    render_main(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);

    // Render overlays
    match app.input_mode {
        InputMode::Search => render_search(frame, app),
        InputMode::VhostSelect => render_vhost_select(frame, app),
        InputMode::MessageView => render_messages(frame, app),
        InputMode::Confirm => render_confirm(frame, app),
        InputMode::Publish => render_publish(frame, app),
        InputMode::Normal => {}
    }
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<&str> = View::all().iter().map(|v| v.name()).collect();
    let selected = View::all().iter().position(|v| *v == app.view).unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" RabbitMQ Monitor "))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(selected);

    frame.render_widget(tabs, area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Overview => render_overview(frame, app, area),
        View::Queues => render_queues(frame, app, area),
        View::Exchanges => render_exchanges(frame, app, area),
        View::Bindings => render_bindings(frame, app, area),
        View::Connections => render_connections(frame, app, area),
        View::Consumers => render_consumers(frame, app, area),
    }
}

fn render_overview(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Overview ");

    if let Some(overview) = &app.overview {
        let mut lines = vec![
            Line::from(vec![
                Span::raw("Cluster: "),
                Span::styled(&overview.cluster_name, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::raw("RabbitMQ: "),
                Span::raw(&overview.rabbitmq_version),
                Span::raw(" | Erlang: "),
                Span::raw(&overview.erlang_version),
            ]),
            Line::from(""),
        ];

        // Object totals
        lines.push(Line::from(Span::styled(
            "Object Totals",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(format!(
            "  Connections: {}  Channels: {}",
            overview.object_totals.connections, overview.object_totals.channels
        )));
        lines.push(Line::from(format!(
            "  Exchanges: {}  Queues: {}  Consumers: {}",
            overview.object_totals.exchanges,
            overview.object_totals.queues,
            overview.object_totals.consumers
        )));
        lines.push(Line::from(""));

        // Queue totals
        if let Some(totals) = &overview.queue_totals {
            lines.push(Line::from(Span::styled(
                "Queue Totals",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(format!(
                "  Messages: {}  Ready: {}  Unacked: {}",
                totals.messages, totals.messages_ready, totals.messages_unacknowledged
            )));
            lines.push(Line::from(""));
        }

        // Message stats
        if let Some(stats) = &overview.message_stats {
            lines.push(Line::from(Span::styled(
                "Message Rates",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(format!(
                "  Publish: {}  Deliver: {}  Ack: {}",
                stats.publish, stats.deliver, stats.ack
            )));
        }

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("Loading...").block(block);
        frame.render_widget(paragraph, area);
    }
}

fn render_queues(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Messages", "Ready", "Unacked", "Consumers", "State"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = get_filtered_items(app, &app.queues, |q| &q.name)
        .iter()
        .enumerate()
        .map(|(i, q)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(q.name.clone()),
                Cell::from(q.messages.to_string()),
                Cell::from(q.messages_ready.to_string()),
                Cell::from(q.messages_unacknowledged.to_string()),
                Cell::from(q.consumers.to_string()),
                Cell::from(q.state_display()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Queues "));

    frame.render_widget(table, area);
}

fn render_exchanges(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Type", "Durable", "Auto-delete", "Internal"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = get_filtered_items(app, &app.exchanges, |e| &e.name)
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(e.display_name()),
                Cell::from(e.exchange_type.clone()),
                Cell::from(if e.durable { "yes" } else { "no" }),
                Cell::from(if e.auto_delete { "yes" } else { "no" }),
                Cell::from(if e.internal { "yes" } else { "no" }),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Exchanges "));

    frame.render_widget(table, area);
}

fn render_bindings(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Source", "Destination", "Type", "Routing Key"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = get_filtered_items(app, &app.bindings, |b| &b.source)
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let source = if b.source.is_empty() {
                "(default)"
            } else {
                &b.source
            };

            Row::new(vec![
                Cell::from(source),
                Cell::from(b.destination.clone()),
                Cell::from(b.destination_type.clone()),
                Cell::from(b.routing_key.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(30),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Bindings "));

    frame.render_widget(table, area);
}

fn render_connections(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "User", "VHost", "Channels", "State"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = get_filtered_items(app, &app.connections, |c| &c.name)
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(c.name.clone()),
                Cell::from(c.user.clone()),
                Cell::from(c.vhost.clone()),
                Cell::from(c.channels.to_string()),
                Cell::from(c.state.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Connections "));

    frame.render_widget(table, area);
}

fn render_consumers(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Tag", "Queue", "Channel", "Ack", "Prefetch"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = get_filtered_items(app, &app.consumers, |c| &c.consumer_tag)
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(c.consumer_tag.clone()),
                Cell::from(c.queue.name.clone()),
                Cell::from(c.channel_details.name.clone()),
                Cell::from(if c.ack_required { "yes" } else { "no" }),
                Cell::from(c.prefetch_count.to_string()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Consumers "));

    frame.render_widget(table, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let help = match app.view {
        View::Overview => "Tab:switch  v:vhost  r:refresh  R:auto  q:quit",
        View::Queues => "Tab:switch  Enter:messages  p:publish  P:purge  d:delete  /:search  q:quit",
        _ => "Tab:switch  /:search  r:refresh  R:auto  q:quit",
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let status_style = if app.connected {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    };

    let status_widget = Paragraph::new(status)
        .style(status_style)
        .block(Block::default().borders(Borders::ALL));

    let help_widget = Paragraph::new(help)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Right);

    frame.render_widget(status_widget, chunks[0]);
    frame.render_widget(help_widget, chunks[1]);
}

fn render_search(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 3, frame.area());
    frame.render_widget(Clear, area);

    let search = Paragraph::new(format!("/{}", app.search_query))
        .block(Block::default().borders(Borders::ALL).title(" Search "));

    frame.render_widget(search, area);
}

fn render_vhost_select(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, (app.vhosts.len() + 2).min(15) as u16, frame.area());
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .vhosts
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(v.name.clone()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Select VHost "));

    frame.render_widget(list, area);
}

fn render_messages(frame: &mut Frame, app: &App) {
    let area = centered_rect(80, 80, frame.area());
    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .margin(1)
        .split(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Queue Messages (Esc to close) ");
    frame.render_widget(block, area);

    // Message list
    let items: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let style = if i == app.message_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            let preview = if m.payload.len() > 50 {
                format!("{}...", &m.payload[..50])
            } else {
                m.payload.clone()
            };
            ListItem::new(format!("[{}] {}", m.routing_key, preview)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Messages "));
    frame.render_widget(list, chunks[0]);

    // Selected message details
    if let Some(msg) = app.messages.get(app.message_selected) {
        let details = vec![
            Line::from(format!("Exchange: {}", msg.exchange)),
            Line::from(format!("Routing Key: {}", msg.routing_key)),
            Line::from(format!("Redelivered: {}", msg.redelivered)),
            Line::from(""),
            Line::from(Span::styled("Payload:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(msg.payload.clone()),
        ];

        let paragraph = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title(" Details "))
            .wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(paragraph, chunks[1]);
    }
}

fn render_confirm(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 7, frame.area());
    frame.render_widget(Clear, area);

    let (title, message) = match &app.confirm_action {
        Some(ConfirmAction::PurgeQueue(name)) => {
            ("Purge Queue", format!("Type '{}' to confirm purging:", name))
        }
        Some(ConfirmAction::DeleteQueue(name)) => {
            ("Delete Queue", format!("Type '{}' to confirm deletion:", name))
        }
        None => ("Confirm", String::new()),
    };

    let content = vec![
        Line::from(message),
        Line::from(""),
        Line::from(format!("> {}_", app.confirm_input)),
    ];

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", title)));

    frame.render_widget(paragraph, area);
}

fn render_publish(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 12, frame.area());
    frame.render_widget(Clear, area);

    let fields = [
        ("Exchange (empty for default)", &app.publish_exchange),
        ("Routing Key", &app.publish_routing_key),
        ("Payload", &app.publish_payload),
    ];

    let content: Vec<Line> = fields
        .iter()
        .enumerate()
        .flat_map(|(i, (label, value))| {
            let style = if i == app.publish_field {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            vec![
                Line::from(Span::styled(*label, style)),
                Line::from(format!(
                    "> {}{}",
                    value,
                    if i == app.publish_field { "_" } else { "" }
                )),
                Line::from(""),
            ]
        })
        .collect();

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Publish Message (Tab to switch, Enter to send) "));

    frame.render_widget(paragraph, area);
}

fn get_filtered_items<'a, T, F>(app: &App, items: &'a [T], _name_fn: F) -> Vec<&'a T>
where
    F: Fn(&T) -> &String,
{
    if app.search_query.is_empty() {
        items.iter().collect()
    } else {
        app.filtered_indices
            .iter()
            .filter_map(|&i| items.get(i))
            .collect()
    }
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
