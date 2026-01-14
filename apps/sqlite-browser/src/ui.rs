use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, View};

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
        View::Tables => render_tables(frame, app, chunks[1]),
        View::Data => render_data(frame, app, chunks[1]),
        View::Schema => render_schema(frame, app, chunks[1]),
        View::Query => render_query(frame, app, chunks[1]),
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let db_name = app.db_path.as_deref().unwrap_or("No database");

    let view_name = match app.view {
        View::Tables => "Tables",
        View::Data => "Data",
        View::Schema => "Schema",
        View::Query => "Query",
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "SQLITE BROWSER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(db_name, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::raw(view_name),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Database "));

    frame.render_widget(header, area);
}

fn render_tables(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .tables
        .iter()
        .enumerate()
        .map(|(i, table)| {
            let style = if i == app.selected_table {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::raw("📋 "),
                Span::styled(&table.name, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    format!("({} rows, {} cols)", table.row_count, table.columns.len()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Tables ({}) ", app.tables.len())),
    );

    frame.render_widget(list, area);
}

fn render_data(frame: &mut Frame, app: &App, area: Rect) {
    if app.data_rows.is_empty() {
        let empty = Paragraph::new("No data")
            .block(Block::default().borders(Borders::ALL).title(" Data "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let visible_columns: Vec<&String> = app
        .data_columns
        .iter()
        .skip(app.column_offset)
        .take(5)
        .collect();

    let header_cells: Vec<Cell> = visible_columns
        .iter()
        .map(|col| {
            Cell::from(Span::styled(
                col.as_str(),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let header = Row::new(header_cells).bottom_margin(1);

    let rows: Vec<Row> = app
        .data_rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let style = if i == app.selected_row {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let cells: Vec<Cell> = row
                .iter()
                .skip(app.column_offset)
                .take(5)
                .map(|cell| Cell::from(truncate(cell, 20)))
                .collect();

            Row::new(cells).style(style)
        })
        .collect();

    let widths: Vec<Constraint> = visible_columns
        .iter()
        .map(|_| Constraint::Min(15))
        .collect();

    let table_name = app
        .selected_table_info()
        .map(|t| t.name.as_str())
        .unwrap_or("Unknown");

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " {} ({} rows) ",
            table_name,
            app.data_rows.len()
        )));

    frame.render_widget(table, area);
}

fn render_schema(frame: &mut Frame, app: &App, area: Rect) {
    let Some(table) = app.selected_table_info() else {
        return;
    };

    let items: Vec<ListItem> = table
        .columns
        .iter()
        .map(|col| {
            let pk_icon = if col.primary_key { "🔑 " } else { "   " };
            let nullable = if col.nullable { "NULL" } else { "NOT NULL" };

            let line = Line::from(vec![
                Span::raw(pk_icon),
                Span::styled(&col.name, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(&col.col_type, Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(nullable, Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Schema: {} ", table.name)),
    );

    frame.render_widget(list, area);
}

fn render_query(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(5)])
        .split(area);

    // Query input
    let border_style = if app.is_editing_query {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let cursor = if app.is_editing_query { "_" } else { "" };
    let query_content = format!("{}{}", app.query, cursor);

    let query = Paragraph::new(query_content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" SQL Query ")
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(query, chunks[0]);

    // Results
    if let Some(ref result) = app.query_result {
        if let Some(ref err) = result.error {
            let error = Paragraph::new(Span::styled(err, Style::default().fg(Color::Red)))
                .block(Block::default().borders(Borders::ALL).title(" Error "));
            frame.render_widget(error, chunks[1]);
        } else if !result.rows.is_empty() {
            let header_cells: Vec<Cell> = result
                .columns
                .iter()
                .map(|col| Cell::from(Span::styled(col, Style::default().fg(Color::Cyan))))
                .collect();

            let header = Row::new(header_cells).bottom_margin(1);

            let rows: Vec<Row> = result
                .rows
                .iter()
                .map(|row| {
                    let cells: Vec<Cell> = row.iter().map(|c| Cell::from(c.as_str())).collect();
                    Row::new(cells)
                })
                .collect();

            let widths: Vec<Constraint> = result
                .columns
                .iter()
                .map(|_| Constraint::Min(15))
                .collect();

            let table = Table::new(rows, widths).header(header).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Results "),
            );

            frame.render_widget(table, chunks[1]);
        } else if let Some(affected) = result.affected_rows {
            let msg = Paragraph::new(format!("{} rows affected", affected))
                .block(Block::default().borders(Borders::ALL).title(" Results "));
            frame.render_widget(msg, chunks[1]);
        }
    } else {
        let placeholder = Paragraph::new("Enter a SQL query and press Ctrl+Enter to execute")
            .block(Block::default().borders(Borders::ALL).title(" Results "))
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, chunks[1]);
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
