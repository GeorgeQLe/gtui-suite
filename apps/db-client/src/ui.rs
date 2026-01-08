use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs},
};

use crate::app::{App, Mode, Pane, View};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_tabs(frame, app, chunks[0]);
    render_main(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    // Render input dialogs
    match &app.mode {
        Mode::QueryInput(query) => render_input_dialog(frame, "SQL Query", query),
        Mode::OpenFile(path) => render_input_dialog(frame, "Open Database", path),
        _ => {}
    }
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["1:Tables", "2:Data", "3:Schema", "4:Query"];
    let selected = match app.view {
        View::Tables => 0,
        View::Data => 1,
        View::Schema => 2,
        View::Query => 3,
        View::Help => 0,
    };

    let db_name = app.db.as_ref()
        .map(|db| db.path.split('/').last().unwrap_or(&db.path).to_string())
        .unwrap_or_else(|| "No database".to_string());

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", db_name)))
        .select(selected)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    frame.render_widget(tabs, area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Tables | View::Data | View::Schema => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(75),
                ])
                .split(area);

            render_table_list(frame, app, chunks[0]);

            match app.view {
                View::Tables => render_table_info(frame, app, chunks[1]),
                View::Data => render_data(frame, app, chunks[1]),
                View::Schema => render_schema(frame, app, chunks[1]),
                _ => {}
            }
        }
        View::Query => render_query(frame, app, area),
        View::Help => render_help(frame, area),
    }
}

fn render_table_list(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.pane == Pane::Tables;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app.tables
        .iter()
        .enumerate()
        .map(|(i, table)| {
            let style = if i == app.selected_table {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("{} ({})", table.name, table.row_count)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Tables "));

    frame.render_widget(list, area);
}

fn render_table_info(frame: &mut Frame, app: &App, area: Rect) {
    let content = if app.tables.is_empty() {
        "No tables found.\n\nPress Ctrl+O to open a database file.".to_string()
    } else if let Some(table) = app.tables.get(app.selected_table) {
        format!(
            "Table: {}\nRows: {}\n\nPress Enter to view data, or:\n  2 - View data\n  3 - View schema\n  4 - Run query",
            table.name,
            table.row_count
        )
    } else {
        "Select a table".to_string()
    };

    let info = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Info "));
    frame.render_widget(info, area);
}

fn render_data(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.pane == Pane::Content;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(format!(" Data: {} ", app.current_table().unwrap_or("?")));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(data) = &app.data {
        if data.rows.is_empty() {
            let empty = Paragraph::new("No data");
            frame.render_widget(empty, inner);
            return;
        }

        let header_cells: Vec<Cell> = data.columns.iter().enumerate().map(|(i, col)| {
            let style = if i == app.selected_col {
                Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::BOLD)
            };
            Cell::from(col.as_str()).style(style)
        }).collect();

        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let visible_height = inner.height.saturating_sub(3) as usize;
        let rows: Vec<Row> = data.rows
            .iter()
            .enumerate()
            .skip(app.data_scroll)
            .take(visible_height)
            .map(|(i, row)| {
                let is_selected = i == app.selected_row;
                let cells: Vec<Cell> = row.iter().enumerate().map(|(j, val)| {
                    let is_cursor = is_selected && j == app.selected_col;
                    let style = if is_cursor {
                        Style::default().bg(Color::Cyan).fg(Color::Black)
                    } else if is_selected {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    };
                    Cell::from(val.as_str()).style(style)
                }).collect();
                Row::new(cells)
            })
            .collect();

        let widths: Vec<Constraint> = data.columns.iter()
            .map(|_| Constraint::Percentage(100 / data.columns.len() as u16))
            .collect();

        let table = Table::new(rows, widths).header(header);
        frame.render_widget(table, inner);
    } else {
        let empty = Paragraph::new("Press Enter on a table to view data");
        frame.render_widget(empty, inner);
    }
}

fn render_schema(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Schema: {} ", app.current_table().unwrap_or("?")));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.schema.is_empty() {
        let empty = Paragraph::new("No schema information");
        frame.render_widget(empty, inner);
        return;
    }

    let header_cells = ["Column", "Type", "Nullable", "PK"].iter().map(|h| {
        Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD))
    });
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = app.schema.iter().map(|col| {
        Row::new(vec![
            Cell::from(col.name.as_str()),
            Cell::from(col.col_type.as_str()),
            Cell::from(if col.nullable { "YES" } else { "NO" }),
            Cell::from(if col.primary_key { "YES" } else { "" }),
        ])
    }).collect();

    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, inner);
}

fn render_query(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(5),
        ])
        .split(area);

    // Results
    let block = Block::default().borders(Borders::ALL).title(" Query Results ");
    let inner = block.inner(chunks[0]);
    frame.render_widget(block, chunks[0]);

    if let Some(result) = &app.query_result {
        let header_cells: Vec<Cell> = result.columns.iter().map(|col| {
            Cell::from(col.as_str()).style(Style::default().add_modifier(Modifier::BOLD))
        }).collect();
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows: Vec<Row> = result.rows.iter().map(|row| {
            Row::new(row.iter().map(|v| Cell::from(v.as_str())).collect::<Vec<_>>())
        }).collect();

        let widths: Vec<Constraint> = result.columns.iter()
            .map(|_| Constraint::Percentage(100 / result.columns.len().max(1) as u16))
            .collect();

        let table = Table::new(rows, widths).header(header);
        frame.render_widget(table, inner);
    } else {
        let hint = Paragraph::new("Press : to enter a SQL query");
        frame.render_widget(hint, inner);
    }

    // History
    let history_text = if app.query_history.is_empty() {
        "No queries yet".to_string()
    } else {
        app.query_history.iter().rev().take(3).cloned().collect::<Vec<_>>().join("\n")
    };

    let history = Paragraph::new(history_text)
        .block(Block::default().borders(Borders::ALL).title(" History "));
    frame.render_widget(history, chunks[1]);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled("Database Client Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Tab            Switch pane"),
        Line::from("  j/k, arrows    Move up/down"),
        Line::from("  h/l            Move left/right (data view)"),
        Line::from("  Enter          View table data"),
        Line::from(""),
        Line::from(Span::styled("Views", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  1              Tables view"),
        Line::from("  2              Data view"),
        Line::from("  3              Schema view"),
        Line::from("  4              Query view"),
        Line::from(""),
        Line::from(Span::styled("Commands", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+O         Open database"),
        Line::from("  :              Enter SQL query"),
        Line::from("  r, F5          Refresh"),
        Line::from("  ?              Show help"),
        Line::from("  q              Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    frame.render_widget(help, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let table_info = if !app.tables.is_empty() {
        format!("Table {}/{}", app.selected_table + 1, app.tables.len())
    } else {
        "No tables".to_string()
    };

    let message = app.message.as_deref()
        .or(app.error.as_deref())
        .unwrap_or("? Help | : Query | Ctrl+O Open | q Quit");

    let style = if app.error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    let status = Paragraph::new(format!(" {} | {} ", table_info, message)).style(style);
    frame.render_widget(status, area);
}

fn render_input_dialog(frame: &mut Frame, title: &str, value: &str) {
    let area = centered_rect(60, 20, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input = Paragraph::new(format!("{}█", value));
    frame.render_widget(input, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup = Layout::default()
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
        .split(popup[1])[1]
}
