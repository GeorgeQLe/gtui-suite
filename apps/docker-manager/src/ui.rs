use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, Tabs},
};

use crate::app::{App, ConfirmAction, Mode, View};
use crate::container::ContainerState;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Tabs
            Constraint::Min(1),     // Main content
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    render_tabs(frame, app, chunks[0]);
    render_content(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    // Render overlays
    match &app.mode {
        Mode::Search(query) => render_search_dialog(frame, query),
        Mode::Confirm(action) => render_confirm_dialog(frame, action),
        Mode::Normal => {}
    }
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Containers", "Images", "Volumes", "Networks"];
    let selected = match app.view {
        View::Containers => 0,
        View::Images => 1,
        View::Volumes => 2,
        View::Networks => 3,
        View::Logs => 0,
        View::Help => 0,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", app.runtime_label())))
        .select(selected)
        .style(Style::default())
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    frame.render_widget(tabs, area);
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Containers => render_containers(frame, app, area),
        View::Images => render_images(frame, app, area),
        View::Volumes => render_volumes(frame, app, area),
        View::Networks => render_networks(frame, app, area),
        View::Logs => render_logs(frame, app, area),
        View::Help => render_help(frame, area),
    }
}

fn render_containers(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Status", "Name", "Image", "Ports", "Status"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app.containers.iter().enumerate().map(|(i, c)| {
        let style = if i == app.container_selected {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            match c.state {
                ContainerState::Running => Style::default().fg(Color::Green),
                ContainerState::Paused => Style::default().fg(Color::Yellow),
                ContainerState::Exited => Style::default().fg(Color::Red),
                _ => Style::default(),
            }
        };

        Row::new(vec![
            c.state.icon().to_string(),
            c.primary_name().to_string(),
            c.image.clone(),
            c.ports_display(),
            c.status.clone(),
        ]).style(style)
    }).collect();

    let title = if app.show_all_containers {
        " Containers (All) "
    } else {
        " Containers (Running) "
    };

    let table = Table::new(rows, [
        Constraint::Length(6),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(20),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}

fn render_images(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Repository:Tag", "ID", "Size", "Created"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app.images.iter().enumerate().map(|(i, img)| {
        let style = if i == app.image_selected {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };

        let created = img.created
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();

        Row::new(vec![
            img.primary_tag().to_string(),
            img.short_id.clone(),
            img.format_size(),
            created,
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Images "));

    frame.render_widget(table, area);
}

fn render_volumes(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Driver", "Mountpoint"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app.volumes.iter().enumerate().map(|(i, v)| {
        let style = if i == app.volume_selected {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };

        Row::new(vec![
            v.name.clone(),
            v.driver.clone(),
            v.mountpoint.clone(),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(30),
        Constraint::Percentage(20),
        Constraint::Percentage(50),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Volumes "));

    frame.render_widget(table, area);
}

fn render_networks(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "ID", "Driver", "Scope", "Containers"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app.networks.iter().enumerate().map(|(i, n)| {
        let style = if i == app.network_selected {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };

        Row::new(vec![
            n.name.clone(),
            n.short_id.clone(),
            n.driver.clone(),
            n.scope.clone(),
            n.containers.len().to_string(),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(25),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(15),
        Constraint::Percentage(20),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Networks "));

    frame.render_widget(table, area);
}

fn render_logs(frame: &mut Frame, app: &App, area: Rect) {
    let title = app.log_container.as_ref()
        .map(|id| format!(" Logs: {} ", &id[..12.min(id.len())]))
        .unwrap_or_else(|| " Logs ".to_string());

    let items: Vec<ListItem> = app.logs.iter()
        .skip(app.logs_scroll)
        .take(area.height as usize - 2)
        .map(|line| ListItem::new(line.as_str()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(list, area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled("Docker Manager Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k, ↑/↓     Move up/down"),
        Line::from("  g/G          Top/bottom"),
        Line::from("  Tab          Next view"),
        Line::from("  Shift+Tab    Previous view"),
        Line::from(""),
        Line::from(Span::styled("Container Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  s            Start container"),
        Line::from("  S            Stop container"),
        Line::from("  r            Restart container"),
        Line::from("  R            Remove container"),
        Line::from("  l            View logs"),
        Line::from("  a            Toggle show all"),
        Line::from(""),
        Line::from(Span::styled("Image Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  R            Remove image"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  /            Search"),
        Line::from("  F5           Refresh"),
        Line::from("  ?            Show this help"),
        Line::from("  Esc          Back / Cancel"),
        Line::from("  q            Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "));

    frame.render_widget(help, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let message = app.message.as_deref()
        .or(app.error.as_deref())
        .unwrap_or("? Help | Tab Switch View | q Quit");

    let style = if app.error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    let status = Paragraph::new(format!(" {} ", message)).style(style);
    frame.render_widget(status, area);
}

fn render_search_dialog(frame: &mut Frame, query: &str) {
    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input_text = format!("{}█", query);
    let input = Paragraph::new(input_text);
    frame.render_widget(input, inner);
}

fn render_confirm_dialog(frame: &mut Frame, action: &ConfirmAction) {
    let area = centered_rect(50, 25, frame.area());
    frame.render_widget(Clear, area);

    let message = match action {
        ConfirmAction::RemoveContainer(id) => {
            format!("Remove container {}?", &id[..12.min(id.len())])
        }
        ConfirmAction::RemoveImage(id) => {
            format!("Remove image {}?", &id[..12.min(id.len())])
        }
        ConfirmAction::StopContainer(id) => {
            format!("Stop container {}?", &id[..12.min(id.len())])
        }
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
