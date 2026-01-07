use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, Mode, View};
use crate::buffer::{format_ascii, format_hex};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);

    match app.view {
        View::Hex => render_hex_view(frame, app, chunks[1]),
        View::Help => render_help(frame, chunks[1]),
    }

    render_status_bar(frame, app, chunks[2]);

    // Overlays
    match &app.mode {
        Mode::Search(query) => render_input_dialog(frame, "Search (hex)", query),
        Mode::Goto(addr) => render_input_dialog(frame, "Go to address (hex)", addr),
        _ => {}
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let title = app.buffer.path.as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "[No file]".to_string());

    let modified = if app.buffer.modified { " [Modified]" } else { "" };
    let mode = match &app.mode {
        Mode::Normal => "",
        Mode::Edit => " [EDIT]",
        Mode::Search(_) => " [SEARCH]",
        Mode::Goto(_) => " [GOTO]",
    };

    let header = Paragraph::new(format!(" {}{}{}", title, modified, mode))
        .style(Style::default().bg(Color::Blue).fg(Color::White));

    frame.render_widget(header, area);
}

fn render_hex_view(frame: &mut Frame, app: &App, area: Rect) {
    let inner = Block::default().borders(Borders::ALL).title(" Hex View ").inner(area);
    frame.render_widget(Block::default().borders(Borders::ALL).title(" Hex View "), area);

    if app.buffer.is_empty() {
        let empty = Paragraph::new("No data loaded. Open a file with command line argument.");
        frame.render_widget(empty, inner);
        return;
    }

    let visible_rows = inner.height as usize;

    let mut lines = Vec::new();
    for row in 0..visible_rows {
        let offset = (app.scroll + row) * app.bytes_per_row;
        if offset >= app.buffer.len() {
            break;
        }

        let mut spans = Vec::new();

        // Address
        spans.push(Span::styled(
            format!("{:08X}  ", offset),
            Style::default().fg(Color::DarkGray),
        ));

        // Hex bytes
        for col in 0..app.bytes_per_row {
            let byte_offset = offset + col;

            if col == 8 {
                spans.push(Span::raw(" "));
            }

            if byte_offset < app.buffer.len() {
                let byte = app.buffer.get(byte_offset).unwrap();
                let is_cursor = byte_offset == app.cursor;

                let style = if is_cursor {
                    if matches!(app.mode, Mode::Edit) {
                        Style::default().bg(Color::Yellow).fg(Color::Black)
                    } else {
                        Style::default().bg(Color::Cyan).fg(Color::Black)
                    }
                } else {
                    Style::default()
                };

                spans.push(Span::styled(format!("{} ", format_hex(byte)), style));
            } else {
                spans.push(Span::raw("   "));
            }
        }

        spans.push(Span::raw(" │ "));

        // ASCII
        for col in 0..app.bytes_per_row {
            let byte_offset = offset + col;

            if byte_offset < app.buffer.len() {
                let byte = app.buffer.get(byte_offset).unwrap();
                let is_cursor = byte_offset == app.cursor;

                let style = if is_cursor {
                    Style::default().bg(Color::Cyan).fg(Color::Black)
                } else {
                    Style::default()
                };

                spans.push(Span::styled(format_ascii(byte).to_string(), style));
            } else {
                spans.push(Span::raw(" "));
            }
        }

        lines.push(Line::from(spans));
    }

    let hex_view = Paragraph::new(lines);
    frame.render_widget(hex_view, inner);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled("Hex Editor Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  h/j/k/l, arrows  Move cursor"),
        Line::from("  PgUp/PgDn        Page up/down"),
        Line::from("  Home/End         Start/end of file"),
        Line::from("  g                Go to address"),
        Line::from(""),
        Line::from(Span::styled("Editing", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  i, Enter         Enter edit mode"),
        Line::from("  0-9, a-f         Edit hex value"),
        Line::from("  Esc              Exit edit mode"),
        Line::from("  u                Undo"),
        Line::from("  Ctrl+R           Redo"),
        Line::from("  Ctrl+S           Save"),
        Line::from(""),
        Line::from(Span::styled("Search", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  /                Search hex pattern"),
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
    let pos = format!("0x{:08X} ({}/{})", app.cursor, app.cursor, app.buffer.len());

    let message = app.message.as_deref()
        .or(app.error.as_deref())
        .unwrap_or("? Help | i Edit | / Search | g Goto | q Quit");

    let style = if app.error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    let status = Paragraph::new(format!(" {} | {} ", pos, message)).style(style);
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
