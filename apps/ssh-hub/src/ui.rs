use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, Tabs},
};

use crate::app::{App, ConfirmAction, HostFormState, Mode, View};

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
        Mode::Search(query) => render_search_dialog(frame, "Search", query),
        Mode::FilterTag(tag) => render_search_dialog(frame, "Filter by Tag", tag),
        Mode::AddHost(form) => render_host_form(frame, "Add Host", form),
        Mode::EditHost(form) => render_host_form(frame, "Edit Host", form),
        Mode::Confirm(action) => render_confirm_dialog(frame, action),
        Mode::Normal => {}
    }
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Hosts", "History", "Snippets"];
    let selected = match app.view {
        View::Hosts => 0,
        View::History => 1,
        View::Snippets => 2,
        View::Help => 0,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" SSH Hub "))
        .select(selected)
        .style(Style::default())
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    frame.render_widget(tabs, area);
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Hosts => render_hosts(frame, app, area),
        View::History => render_history(frame, app, area),
        View::Snippets => render_snippets(frame, app, area),
        View::Help => render_help(frame, area),
    }
}

fn render_hosts(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Host", "User", "Tags", "Last Connected"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app.filtered_hosts.iter().enumerate().map(|(i, &idx)| {
        let host = &app.hosts[idx];
        let style = if i == app.selected {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };

        Row::new(vec![
            host.name.clone(),
            host.connection_string(),
            host.user.clone().unwrap_or_default(),
            host.tags_display(),
            host.last_connected_display(),
        ]).style(style)
    }).collect();

    let title = if let Some(tag) = &app.tag_filter {
        format!(" Hosts [tag: {}] ", tag)
    } else if !app.search_query.is_empty() {
        format!(" Hosts [search: {}] ", app.search_query)
    } else {
        " Hosts ".to_string()
    };

    let table = Table::new(rows, [
        Constraint::Percentage(20),
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(table, area);
}

fn render_history(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.filtered_hosts.iter().enumerate()
        .filter_map(|(i, &idx)| {
            let host = &app.hosts[idx];
            host.last_connected.map(|_| {
                let style = if i == app.selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                ListItem::new(format!(
                    "{} - {} ({})",
                    host.name,
                    host.connection_string(),
                    host.last_connected_display()
                )).style(style)
            })
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Recent Connections "));

    frame.render_widget(list, area);
}

fn render_snippets(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.snippets.iter().enumerate().map(|(i, snippet)| {
        let style = if i == app.snippet_selected {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };

        let scope = if snippet.host_id.is_some() { "host" } else { "global" };
        ListItem::new(format!("[{}] {} - {}", scope, snippet.name, snippet.command))
            .style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Command Snippets "));

    frame.render_widget(list, area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled("SSH Hub Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k, ↑/↓     Move up/down"),
        Line::from("  g/G          Top/bottom"),
        Line::from("  Tab          Switch views"),
        Line::from("  h            History view"),
        Line::from("  s            Snippets view"),
        Line::from(""),
        Line::from(Span::styled("Host Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Enter        Connect to host"),
        Line::from("  a            Add new host"),
        Line::from("  e            Edit host"),
        Line::from("  d            Delete host"),
        Line::from(""),
        Line::from(Span::styled("Search & Filter", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  /            Search hosts"),
        Line::from("  t            Filter by tag"),
        Line::from("  Esc          Clear filter"),
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
        .unwrap_or("? Help | / Search | t Tag filter | a Add | Enter Connect");

    let style = if app.error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    let count = format!(" {} hosts ", app.filtered_hosts.len());
    let status = Paragraph::new(format!("{} | {}", count, message)).style(style);
    frame.render_widget(status, area);
}

fn render_search_dialog(frame: &mut Frame, title: &str, query: &str) {
    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input_text = format!("{}█", query);
    let input = Paragraph::new(input_text);
    frame.render_widget(input, inner);
}

fn render_host_form(frame: &mut Frame, title: &str, form: &HostFormState) {
    let area = centered_rect(60, 70, frame.area());
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
            (0..HostFormState::field_count())
                .map(|_| Constraint::Length(2))
                .collect::<Vec<_>>()
        )
        .split(inner);

    for i in 0..HostFormState::field_count() {
        let label = HostFormState::field_label(i);
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
        ConfirmAction::DeleteHost(id) => format!("Delete host {}?", &id[..8.min(id.len())]),
        ConfirmAction::Connect(id) => format!("Connect to {}?", &id[..8.min(id.len())]),
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
