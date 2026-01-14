use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{App, Mode, View};

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
        View::Edit => render_editor(frame, app, chunks[1]),
        View::Preview => render_preview(frame, app, chunks[1]),
        View::Split => render_split(frame, app, chunks[1]),
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let mode_str = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
    };

    let mode_color = match app.mode {
        Mode::Normal => Color::Blue,
        Mode::Insert => Color::Green,
    };

    let modified = if app.modified { " [+]" } else { "" };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "MARKDOWN EDITOR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(mode_str, Style::default().fg(mode_color)),
        Span::raw(" | "),
        Span::styled(
            format!("{:?}", app.view),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(modified),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Editor "));

    frame.render_widget(header, area);
}

fn render_editor(frame: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = app
        .lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line_num = format!("{:>4} ", i + 1);
            let is_current = i == app.cursor_line;

            let line_style = if is_current {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Line::styled(
                format!("{}{}", line_num, line),
                line_style,
            )
        })
        .collect();

    let border_style = if app.mode == Mode::Insert {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let editor = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Source ")
                .border_style(border_style),
        )
        .scroll((app.scroll_offset as u16, 0));

    frame.render_widget(editor, area);

    // Show cursor in insert mode
    if app.mode == Mode::Insert {
        let cursor_x = area.x + 1 + 5 + app.cursor_col as u16;
        let cursor_y = area.y + 1 + (app.cursor_line - app.scroll_offset) as u16;
        if cursor_x < area.right() && cursor_y < area.bottom() {
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let preview_lines = render_markdown(&app.lines);

    let preview = Paragraph::new(preview_lines)
        .block(Block::default().borders(Borders::ALL).title(" Preview "))
        .wrap(Wrap { trim: false });

    frame.render_widget(preview, area);
}

fn render_split(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_editor(frame, app, chunks[0]);
    render_preview(frame, app, chunks[1]);
}

fn render_markdown(lines: &[String]) -> Vec<Line<'static>> {
    lines
        .iter()
        .map(|line| {
            let trimmed = line.trim();

            if trimmed.starts_with("# ") {
                Line::styled(
                    trimmed.to_string(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else if trimmed.starts_with("## ") {
                Line::styled(
                    trimmed.to_string(),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
            } else if trimmed.starts_with("### ") {
                Line::styled(
                    trimmed.to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                Line::styled(
                    format!("  • {}", &trimmed[2..]),
                    Style::default().fg(Color::White),
                )
            } else if trimmed.starts_with("> ") {
                Line::styled(
                    format!("│ {}", &trimmed[2..]),
                    Style::default().fg(Color::DarkGray),
                )
            } else if trimmed.starts_with("```") {
                Line::styled(
                    "───────────────────────────────".to_string(),
                    Style::default().fg(Color::DarkGray),
                )
            } else if trimmed.starts_with("---") {
                Line::styled(
                    "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string(),
                    Style::default().fg(Color::DarkGray),
                )
            } else if trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
                && trimmed.contains(". ")
            {
                Line::styled(line.to_string(), Style::default().fg(Color::Yellow))
            } else {
                // Handle inline formatting (simplified)
                let processed = line
                    .replace("**", "")
                    .replace("*", "")
                    .replace("`", "");
                Line::styled(processed, Style::default().fg(Color::White))
            }
        })
        .collect()
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
