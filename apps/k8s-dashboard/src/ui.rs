use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, ConfirmAction, InputMode, View};
use crate::models::{PodPhase, ResourceType};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(area);

    render_main(frame, app, chunks[0]);
    render_status(frame, app, chunks[1]);

    // Render overlays
    match app.input_mode {
        InputMode::Search => render_search(frame, app),
        InputMode::NamespaceSelect => render_namespace_select(frame, app),
        InputMode::ContextSelect => render_context_select(frame, app),
        InputMode::ScaleDialog => render_scale_dialog(frame, app),
        InputMode::Confirm => render_confirm(frame, app),
        _ => {}
    }
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Resources => {
            if app.show_sidebar {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(20), Constraint::Min(50)])
                    .split(area);

                render_sidebar(frame, app, chunks[0]);
                render_resources(frame, app, chunks[1]);
            } else {
                render_resources(frame, app, area);
            }
        }
        View::Details => {
            render_details(frame, app, area);
        }
        View::Logs => {
            render_logs(frame, app, area);
        }
        View::Yaml => {
            render_yaml(frame, app, area);
        }
    }
}

fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = ResourceType::all()
        .iter()
        .map(|rt| {
            let style = if *rt == app.resource_type {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(rt.as_str()).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Resources "));

    frame.render_widget(list, area);
}

fn render_resources(frame: &mut Frame, app: &App, area: Rect) {
    match app.resource_type {
        ResourceType::Pods => render_pods(frame, app, area),
        ResourceType::Deployments => render_deployments(frame, app, area),
        ResourceType::Services => render_services(frame, app, area),
        ResourceType::Nodes => render_nodes(frame, app, area),
        ResourceType::Events => render_events(frame, app, area),
        ResourceType::Namespaces => render_namespaces(frame, app, area),
        _ => {
            let empty = Paragraph::new("Not implemented")
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(empty, area);
        }
    }
}

fn render_pods(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Ready", "Status", "Restarts", "Age"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .pods
        .iter()
        .enumerate()
        .map(|(i, pod)| {
            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let status_style = match pod.phase {
                PodPhase::Running => Style::default().fg(Color::Green),
                PodPhase::Pending => Style::default().fg(Color::Yellow),
                PodPhase::Failed => Style::default().fg(Color::Red),
                PodPhase::Succeeded => Style::default().fg(Color::Cyan),
                PodPhase::Unknown => Style::default().fg(Color::Gray),
            };

            Row::new(vec![
                Cell::from(pod.name.clone()),
                Cell::from(pod.ready.clone()),
                Cell::from(format!("{} {}", pod.phase.icon(), pod.phase.as_str())).style(status_style),
                Cell::from(pod.restarts.to_string()),
                Cell::from(pod.age_display()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(" Pods ({}) ", app.pods.len())));

    frame.render_widget(table, area);
}

fn render_deployments(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Ready", "Up-to-date", "Available", "Age"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .deployments
        .iter()
        .enumerate()
        .map(|(i, deploy)| {
            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(deploy.name.clone()),
                Cell::from(deploy.ready.clone()),
                Cell::from(deploy.up_to_date.to_string()),
                Cell::from(deploy.available.to_string()),
                Cell::from(deploy.age_display()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(" Deployments ({}) ", app.deployments.len())));

    frame.render_widget(table, area);
}

fn render_services(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Type", "Cluster-IP", "External-IP", "Ports"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .services
        .iter()
        .enumerate()
        .map(|(i, svc)| {
            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(svc.name.clone()),
                Cell::from(svc.service_type.clone()),
                Cell::from(svc.cluster_ip.clone()),
                Cell::from(svc.external_ip.clone().unwrap_or_else(|| "<none>".to_string())),
                Cell::from(svc.ports.join(", ")),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(" Services ({}) ", app.services.len())));

    frame.render_widget(table, area);
}

fn render_nodes(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Status", "Roles", "Age", "Version"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(node.name.clone()),
                Cell::from(node.status.as_str()),
                Cell::from(node.roles.join(",")),
                Cell::from(format!("{}d", node.age.as_secs() / 86400)),
                Cell::from(node.version.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(" Nodes ({}) ", app.nodes.len())));

    frame.render_widget(table, area);
}

fn render_events(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Type", "Reason", "Object", "Message"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .events
        .iter()
        .enumerate()
        .map(|(i, event)| {
            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let type_style = if event.type_ == "Warning" {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            };

            Row::new(vec![
                Cell::from(event.type_.clone()).style(type_style),
                Cell::from(event.reason.clone()),
                Cell::from(event.object.clone()),
                Cell::from(event.message.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(50),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(" Events ({}) ", app.events.len())));

    frame.render_widget(table, area);
}

fn render_namespaces(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Status"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .namespaces
        .iter()
        .enumerate()
        .map(|(i, ns)| {
            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(ns.name.clone()),
                Cell::from(ns.status.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(" Namespaces ({}) ", app.namespaces.len())));

    frame.render_widget(table, area);
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(pod) = &app.current_pod {
        let info = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&pod.name),
            ]),
            Line::from(vec![
                Span::styled("Namespace: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&pod.namespace),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} {}", pod.phase.icon(), pod.phase.as_str())),
            ]),
            Line::from(vec![
                Span::styled("Ready: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&pod.ready),
            ]),
            Line::from(vec![
                Span::styled("Node: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&pod.node),
            ]),
            Line::from(vec![
                Span::styled("Restarts: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(pod.restarts.to_string()),
            ]),
            Line::from(vec![
                Span::styled("CPU: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(pod.cpu_display()),
            ]),
            Line::from(vec![
                Span::styled("Memory: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(pod.memory_display()),
            ]),
        ];

        let paragraph = Paragraph::new(info)
            .block(Block::default().borders(Borders::ALL).title(" Pod Details "));
        frame.render_widget(paragraph, area);
    } else if let Some(deploy) = &app.current_deployment {
        let info = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&deploy.name),
            ]),
            Line::from(vec![
                Span::styled("Namespace: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&deploy.namespace),
            ]),
            Line::from(vec![
                Span::styled("Ready: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&deploy.ready),
            ]),
            Line::from(vec![
                Span::styled("Available: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(deploy.available.to_string()),
            ]),
        ];

        let paragraph = Paragraph::new(info)
            .block(Block::default().borders(Borders::ALL).title(" Deployment Details "));
        frame.render_widget(paragraph, area);
    } else {
        let empty = Paragraph::new("No resource selected")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, area);
    }
}

fn render_logs(frame: &mut Frame, app: &App, area: Rect) {
    let content = app.logs.as_deref().unwrap_or("No logs available");
    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Logs "))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn render_yaml(frame: &mut Frame, app: &App, area: Rect) {
    let content = app.yaml.as_deref().unwrap_or("No YAML available");
    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" YAML "))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let style = Style::default().bg(Color::DarkGray);
    let paragraph = Paragraph::new(format!(" {} ", status)).style(style);
    frame.render_widget(paragraph, area);
}

fn render_search(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 3, frame.area());
    frame.render_widget(Clear, area);

    let search = Paragraph::new(format!("/{}", app.search_query))
        .block(Block::default().borders(Borders::ALL).title(" Search "));
    frame.render_widget(search, area);
}

fn render_namespace_select(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 30, frame.area());
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .namespaces
        .iter()
        .enumerate()
        .map(|(i, ns)| {
            let style = if i == app.active_namespace {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(ns.name.clone()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Select Namespace "));
    frame.render_widget(list, area);
}

fn render_context_select(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 20, frame.area());
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .contexts
        .iter()
        .enumerate()
        .map(|(i, ctx)| {
            let style = if i == app.active_context {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(format!("{} ({})", ctx.name, ctx.cluster)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Select Context "));
    frame.render_widget(list, area);
}

fn render_scale_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(30, 5, frame.area());
    frame.render_widget(Clear, area);

    let content = format!("Replicas: {}\n\nj/k or arrows to adjust, Enter to confirm", app.scale_value);
    let dialog = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Scale Deployment "))
        .alignment(Alignment::Center);
    frame.render_widget(dialog, area);
}

fn render_confirm(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 5, frame.area());
    frame.render_widget(Clear, area);

    let message = match &app.confirm_action {
        Some(ConfirmAction::DeletePod(name)) => format!("Delete pod {}?", name),
        Some(ConfirmAction::RestartDeployment(name)) => format!("Restart deployment {}?", name),
        Some(ConfirmAction::CordonNode(name)) => format!("Cordon node {}?", name),
        None => String::new(),
    };

    let content = format!("{}\n\n(y) Yes  (n) No", message);
    let dialog = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Confirm "))
        .alignment(Alignment::Center);
    frame.render_widget(dialog, area);
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
