use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

use crate::app::{App, Mode, View};
use crate::data::calculate_column_widths;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    match app.view {
        View::Table => render_table(frame, app, chunks[0]),
        View::Help => render_help(frame, chunks[0]),
    }

    render_status_bar(frame, app, chunks[1]);

    match &app.mode {
        Mode::Search(query) => render_input_dialog(frame, "Search", query),
        Mode::Filter(query) => render_input_dialog(frame, &format!("Filter column {}", app.cursor_col + 1), query),
        _ => {}
    }
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(
        app.data.path.as_ref()
            .map(|p| format!(" {} ", p.display()))
            .unwrap_or_else(|| " CSV Viewer ".to_string())
    );
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.data.col_count() == 0 {
        let empty = Paragraph::new("No data loaded. Open a CSV file with command line argument.");
        frame.render_widget(empty, inner);
        return;
    }

    let col_widths = calculate_column_widths(&app.data, inner.width);
    let constraints: Vec<Constraint> = col_widths.iter().map(|&w| Constraint::Length(w)).collect();

    // Header
    let header_cells: Vec<Cell> = app.data.headers.iter().enumerate().map(|(i, h)| {
        let style = if i == app.cursor_col {
            Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
        } else if app.sort_column == Some(i) {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::BOLD)
        };

        let label = if app.sort_column == Some(i) {
            format!("{} {}", h, if app.sort_ascending { "▲" } else { "▼" })
        } else {
            h.clone()
        };

        Cell::from(label).style(style)
    }).collect();

    let header = Row::new(header_cells).height(1).bottom_margin(1);

    // Rows
    let visible_height = inner.height.saturating_sub(3) as usize;
    let rows: Vec<Row> = (app.scroll_row..app.scroll_row + visible_height)
        .filter_map(|display_idx| {
            let actual_row = app.get_display_row(display_idx)?;
            let is_selected = display_idx == app.cursor_row;

            let cells: Vec<Cell> = (0..app.data.col_count()).map(|col| {
                let content = app.data.get_cell(actual_row, col).unwrap_or("");
                let is_cursor = is_selected && col == app.cursor_col;
                let is_search_match = app.search_results.contains(&(actual_row, col));

                let style = if is_cursor {
                    Style::default().bg(Color::Cyan).fg(Color::Black)
                } else if is_search_match {
                    Style::default().bg(Color::Yellow).fg(Color::Black)
                } else if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                Cell::from(content).style(style)
            }).collect();

            Some(Row::new(cells))
        })
        .collect();

    let table = Table::new(rows, constraints).header(header);
    frame.render_widget(table, inner);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled("CSV Viewer Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  h/j/k/l, arrows  Move cursor"),
        Line::from("  PgUp/PgDn        Page up/down"),
        Line::from("  Home/End         First/last row"),
        Line::from(""),
        Line::from(Span::styled("Data Operations", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  /                Search all columns"),
        Line::from("  n/N              Next/prev search result"),
        Line::from("  f                Filter current column"),
        Line::from("  s                Sort by column"),
        Line::from("  Esc              Clear filter/search"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  ?                Show help"),
        Line::from("  q                Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    frame.render_widget(help, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let pos = format!("Row {}/{} Col {}/{}",
        app.cursor_row + 1, app.visible_rows(),
        app.cursor_col + 1, app.data.col_count());

    let mode_str = match &app.mode {
        Mode::Sort => " [SORT]",
        _ => "",
    };

    let message = app.message.as_deref()
        .or(app.error.as_deref())
        .unwrap_or("? Help | / Search | f Filter | s Sort | q Quit");

    let style = if app.error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    let status = Paragraph::new(format!(" {}{} | {} ", pos, mode_str, message)).style(style);
    frame.render_widget(status, area);
}

fn render_input_dialog(frame: &mut Frame, title: &str, value: &str) {
    let area = centered_rect(50, 20, frame.area());
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
