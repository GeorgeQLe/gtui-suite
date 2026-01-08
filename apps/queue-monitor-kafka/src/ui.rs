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
        InputMode::ClusterSelect => render_cluster_select(frame, app),
        InputMode::TopicDetail => render_topic_detail(frame, app),
        InputMode::GroupDetail => render_group_detail(frame, app),
        InputMode::Confirm => render_confirm(frame, app),
        InputMode::Normal => {}
    }
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<&str> = View::all().iter().map(|v| v.name()).collect();
    let selected = View::all().iter().position(|v| *v == app.view).unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Kafka Monitor "))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(selected);

    frame.render_widget(tabs, area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Topics => render_topics(frame, app, area),
        View::Partitions => render_partitions(frame, app, area),
        View::ConsumerGroups => render_consumer_groups(frame, app, area),
        View::Lag => render_lag(frame, app, area),
        View::Brokers => render_brokers(frame, app, area),
    }
}

fn render_topics(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Partitions", "Replication", "Internal"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = get_filtered_items(app, &app.topics)
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(t.name.clone()),
                Cell::from(t.partitions.to_string()),
                Cell::from(t.replication_factor.to_string()),
                Cell::from(if t.internal { "yes" } else { "no" }),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Topics "));

    frame.render_widget(table, area);
}

fn render_partitions(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Topic", "Partition", "Leader", "Replicas", "ISR", "Messages"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .partitions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(p.topic.clone()),
                Cell::from(p.partition.to_string()),
                Cell::from(p.leader.to_string()),
                Cell::from(format!("{:?}", p.replicas)),
                Cell::from(format!("{:?}", p.isr)),
                Cell::from(p.message_count().to_string()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(18),
            Constraint::Percentage(18),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Partitions "));

    frame.render_widget(table, area);
}

fn render_consumer_groups(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "State", "Members", "Protocol"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = get_filtered_items(app, &app.consumer_groups)
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let state_style = match g.state {
                crate::models::GroupState::Stable => Style::default().fg(Color::Green),
                crate::models::GroupState::PreparingRebalance
                | crate::models::GroupState::CompletingRebalance => {
                    Style::default().fg(Color::Yellow)
                }
                crate::models::GroupState::Dead => Style::default().fg(Color::Red),
                _ => Style::default(),
            };

            Row::new(vec![
                Cell::from(g.name.clone()),
                Cell::from(Span::styled(g.state.as_str(), state_style)),
                Cell::from(g.members.len().to_string()),
                Cell::from(g.protocol_type.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Consumer Groups "));

    frame.render_widget(table, area);
}

fn render_lag(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    // Summary
    let total_lag = app.total_lag();
    let lag_style = if total_lag > app.config.display.lag_alert_threshold {
        Style::default().fg(Color::Red)
    } else if total_lag > 1000 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };

    let summary = Paragraph::new(Line::from(vec![
        Span::raw("Total Lag: "),
        Span::styled(format!("{}", total_lag), lag_style),
        Span::raw(format!(" | Groups: {} | ", app.consumer_groups.len())),
        Span::raw(format!("Threshold: {}", app.config.display.lag_alert_threshold)),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Lag Summary "));

    frame.render_widget(summary, chunks[0]);

    // Lag table
    let header = Row::new(vec!["Group", "Topic", "Partition", "Current", "End", "Lag"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = get_filtered_items(app, &app.lag_data)
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let lag_style = if l.lag > app.config.display.lag_alert_threshold {
                Style::default().fg(Color::Red)
            } else if l.lag > 1000 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            };

            Row::new(vec![
                Cell::from(l.group.clone()),
                Cell::from(l.topic.clone()),
                Cell::from(l.partition.to_string()),
                Cell::from(l.current_offset.to_string()),
                Cell::from(l.log_end_offset.to_string()),
                Cell::from(Span::styled(l.lag.to_string(), lag_style)),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(12),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Consumer Lag "));

    frame.render_widget(table, chunks[1]);
}

fn render_brokers(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["ID", "Host", "Port", "Rack", "Controller"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .brokers
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(b.id.to_string()),
                Cell::from(b.host.clone()),
                Cell::from(b.port.to_string()),
                Cell::from(b.rack.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(if b.is_controller { "yes" } else { "no" }),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Brokers "));

    frame.render_widget(table, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let help = "Tab:switch  Enter:details  d:delete  l:lag  /:search  r:refresh  R:auto  q:quit";

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

fn render_cluster_select(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, (app.config.clusters.len() + 2).min(15) as u16, frame.area());
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .config
        .clusters
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            let marker = if c.name == app.current_cluster {
                " *"
            } else {
                ""
            };
            ListItem::new(format!("{}{}", c.name, marker)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Select Cluster "));

    frame.render_widget(list, area);
}

fn render_topic_detail(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 60, frame.area());
    frame.render_widget(Clear, area);

    let topic_name = app.selected_topic.as_deref().unwrap_or("Unknown");
    let topic = app.topics.iter().find(|t| t.name == topic_name);

    let content = if let Some(t) = topic {
        vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&t.name),
            ]),
            Line::from(vec![
                Span::styled("Partitions: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(t.partitions.to_string()),
            ]),
            Line::from(vec![
                Span::styled(
                    "Replication Factor: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(t.replication_factor.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Internal: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(if t.internal { "yes" } else { "no" }),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Press Esc to close",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        vec![Line::from("Topic not found")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(format!(" Topic: {} ", topic_name)));

    frame.render_widget(paragraph, area);
}

fn render_group_detail(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 60, frame.area());
    frame.render_widget(Clear, area);

    let group_name = app.selected_group.as_deref().unwrap_or("Unknown");
    let group = app.consumer_groups.iter().find(|g| g.name == group_name);

    let content = if let Some(g) = group {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&g.name),
            ]),
            Line::from(vec![
                Span::styled("State: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(g.state.as_str()),
            ]),
            Line::from(vec![
                Span::styled("Members: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(g.members.len().to_string()),
            ]),
            Line::from(""),
        ];

        for member in &g.members {
            lines.push(Line::from(Span::styled(
                format!("Member: {}", member.client_id),
                Style::default().add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(format!("  Host: {}", member.client_host)));
            for assignment in &member.assignments {
                lines.push(Line::from(format!(
                    "  {} -> {:?}",
                    assignment.topic, assignment.partitions
                )));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Press Esc to close",
            Style::default().fg(Color::DarkGray),
        )));

        lines
    } else {
        vec![Line::from("Group not found")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(format!(" Consumer Group: {} ", group_name)));

    frame.render_widget(paragraph, area);
}

fn render_confirm(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 7, frame.area());
    frame.render_widget(Clear, area);

    let (title, message) = match &app.confirm_action {
        Some(ConfirmAction::DeleteTopic(name)) => {
            ("Delete Topic", format!("Type '{}' to confirm deletion:", name))
        }
        Some(ConfirmAction::ResetOffsets(group, topic)) => (
            "Reset Offsets",
            format!("Reset offsets for {} on {}?", group, topic),
        ),
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

fn get_filtered_items<'a, T>(app: &App, items: &'a [T]) -> Vec<&'a T> {
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
