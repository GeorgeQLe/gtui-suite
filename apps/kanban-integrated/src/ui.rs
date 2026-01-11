use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, InputMode, View};
use crate::models::{Priority, SyncStatus};

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
        InputMode::AddCard => render_add_card(frame, app),
        _ => {}
    }
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Board => render_board(frame, app, area),
        View::CardDetail => render_card_detail(frame, app, area),
        View::Sources => render_sources(frame, app, area),
        View::Conflicts => render_conflicts(frame, app, area),
        View::Import => render_import(frame, app, area),
    }
}

fn render_board(frame: &mut Frame, app: &App, area: Rect) {
    let column_count = app.board.columns.len();
    if column_count == 0 {
        let empty = Paragraph::new("No columns. Press 'n' to add a card.")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, area);
        return;
    }

    let column_constraints: Vec<Constraint> = app
        .board
        .columns
        .iter()
        .map(|_| Constraint::Ratio(1, column_count as u32))
        .collect();

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(column_constraints)
        .split(area);

    for (col_idx, column) in app.board.columns.iter().enumerate() {
        let is_selected_column = col_idx == app.selected_column;

        let border_style = if is_selected_column {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let title = if let Some(limit) = column.limit {
            format!(" {} ({}/{}) ", column.name, column.cards.len(), limit)
        } else {
            format!(" {} ({}) ", column.name, column.cards.len())
        };

        let items: Vec<ListItem> = column
            .cards
            .iter()
            .enumerate()
            .map(|(card_idx, card)| {
                let is_selected = is_selected_column && card_idx == app.selected_card;
                let style = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let priority_icon = card.priority.icon();
                let priority_color = match card.priority {
                    Priority::Critical => Color::Red,
                    Priority::High => Color::Yellow,
                    Priority::Medium => Color::White,
                    Priority::Low => Color::Gray,
                };

                let linked = if card.is_linked() { " 🔗" } else { "" };

                let (done, total) = card.checklist_progress();
                let checklist = if total > 0 {
                    format!(" [{}/{}]", done, total)
                } else {
                    String::new()
                };

                let content = Line::from(vec![
                    Span::styled(format!("{} ", priority_icon), Style::default().fg(priority_color)),
                    Span::raw(&card.title),
                    Span::styled(linked, Style::default().fg(Color::Cyan)),
                    Span::styled(checklist, Style::default().fg(Color::Gray)),
                ]);

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(title),
        );

        frame.render_widget(list, columns[col_idx]);
    }
}

fn render_card_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(card) = &app.current_card else {
        let empty = Paragraph::new("No card selected")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Min(5),
        ])
        .split(area);

    // Card info
    let priority_color = match card.priority {
        Priority::Critical => Color::Red,
        Priority::High => Color::Yellow,
        Priority::Medium => Color::White,
        Priority::Low => Color::Gray,
    };

    let info = vec![
        Line::from(vec![
            Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&card.title),
        ]),
        Line::from(vec![
            Span::styled("Priority: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{} {}", card.priority.icon(), card.priority.as_str()),
                Style::default().fg(priority_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("Labels: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(card.labels.join(", ")),
        ]),
        Line::from(vec![
            Span::styled("Assignee: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(card.assignee.as_deref().unwrap_or("Unassigned")),
        ]),
    ];

    let info_widget = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title(" Card Details "));
    frame.render_widget(info_widget, chunks[0]);

    // Description
    let desc = if card.description.is_empty() {
        "No description"
    } else {
        &card.description
    };

    let desc_widget = Paragraph::new(desc)
        .block(Block::default().borders(Borders::ALL).title(" Description "))
        .wrap(Wrap { trim: true });
    frame.render_widget(desc_widget, chunks[1]);

    // Checklist
    let checklist_items: Vec<ListItem> = card
        .checklist
        .iter()
        .map(|item| {
            let icon = if item.done { "☑" } else { "☐" };
            let style = if item.done {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            ListItem::new(format!("{} {}", icon, item.text)).style(style)
        })
        .collect();

    let (done, total) = card.checklist_progress();
    let checklist_title = if total > 0 {
        format!(" Checklist ({}/{}) ", done, total)
    } else {
        " Checklist ".to_string()
    };

    let checklist_widget = List::new(checklist_items)
        .block(Block::default().borders(Borders::ALL).title(checklist_title));
    frame.render_widget(checklist_widget, chunks[2]);
}

fn render_sources(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Type", "Name", "Status", "Last Sync"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .board
        .sources
        .iter()
        .enumerate()
        .map(|(i, source)| {
            let style = if i == app.selected_source {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let status_color = match source.sync_status {
                SyncStatus::Synced => Color::Green,
                SyncStatus::LocalChanges => Color::Yellow,
                SyncStatus::RemoteChanges => Color::Cyan,
                SyncStatus::Conflict => Color::Red,
            };

            Row::new(vec![
                Cell::from(format!("{} {}", source.source_type.icon(), source.source_type.as_str())),
                Cell::from(source.name.clone()),
                Cell::from(format!("{} {}", source.sync_status.icon(), source.sync_status.as_str()))
                    .style(Style::default().fg(status_color)),
                Cell::from(source.last_sync_display()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" External Sources ({}) ", app.board.sources.len())),
    );

    frame.render_widget(table, area);
}

fn render_conflicts(frame: &mut Frame, app: &App, area: Rect) {
    if app.conflicts.is_empty() {
        let empty = Paragraph::new("No conflicts to resolve")
            .block(Block::default().borders(Borders::ALL).title(" Conflicts "));
        frame.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec!["Card", "Field", "Local", "Remote"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .conflicts
        .iter()
        .enumerate()
        .map(|(i, conflict)| {
            let style = if i == app.selected_conflict {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(conflict.local_title.clone()),
                Cell::from(conflict.field.clone()),
                Cell::from(conflict.local_value.clone()),
                Cell::from(conflict.remote_value.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(27),
            Constraint::Percentage(28),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Conflicts ({}) ", app.conflicts.len())),
    );

    frame.render_widget(table, area);
}

fn render_import(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .board
        .sources
        .iter()
        .enumerate()
        .map(|(i, source)| {
            let style = if i == app.selected_source {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(format!(
                "{} {} - {}",
                source.source_type.icon(),
                source.source_type.as_str(),
                source.name
            ))
            .style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Import from Source "));

    frame.render_widget(list, area);
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

fn render_add_card(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 3, frame.area());
    frame.render_widget(Clear, area);

    let input = Paragraph::new(app.input_buffer.as_str())
        .block(Block::default().borders(Borders::ALL).title(" New Card Title "));

    frame.render_widget(input, area);
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
