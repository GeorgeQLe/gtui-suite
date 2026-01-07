use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::app::{App, Mode};
use crate::log_entry::LogLevel;

pub fn render(frame: &mut Frame, app: &mut App) {
    app.viewport_height = frame.area().height;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Status bar
            Constraint::Min(1),     // Log content
            Constraint::Length(1),  // Input/message bar
        ])
        .split(frame.area());

    render_status_bar(frame, app, chunks[0]);
    render_log_content(frame, app, chunks[1]);
    render_input_bar(frame, app, chunks[2]);

    // Render overlays
    if app.mode == Mode::Help {
        render_help(frame);
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let file_info = app.file_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "Demo Mode".to_string());

    let visible = app.visible_entries();
    let total = app.entries.len();
    let filtered = if visible.len() != total {
        format!(" ({}/{})", visible.len(), total)
    } else {
        format!(" ({})", total)
    };

    let follow_indicator = if app.follow_mode { " [FOLLOW]" } else { "" };

    let filter_indicator = app.level_filter
        .map(|l| format!(" [{}+]", l.label()))
        .unwrap_or_default();

    let bookmark_count = if !app.bookmarks.is_empty() {
        format!(" [{}B]", app.bookmarks.len())
    } else {
        String::new()
    };

    let status = format!(
        " {} {} {}{}{}{}",
        file_info,
        filtered,
        follow_indicator,
        filter_indicator,
        bookmark_count,
        if !app.search_query.is_empty() {
            format!(" [/{}]", app.search_query)
        } else {
            String::new()
        }
    );

    let status_bar = Paragraph::new(status)
        .style(Style::default().bg(Color::Blue).fg(Color::White));

    frame.render_widget(status_bar, area);
}

fn render_log_content(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::NONE);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Update viewport height for scrolling calculations first
    app.viewport_height = area.height;

    let visible = app.visible_entries();
    let viewport_height = inner.height as usize;

    // Calculate line number width
    let max_line = visible.last().map(|e| e.line_number).unwrap_or(1);
    let line_num_width = if app.show_line_numbers {
        max_line.to_string().len() + 2
    } else {
        0
    };

    // Render visible lines
    for (i, entry) in visible.iter().skip(app.scroll_offset).take(viewport_height).enumerate() {
        let y = inner.y + i as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let is_selected = app.scroll_offset + i == app.selected;
        let is_bookmarked = app.bookmarks.contains(&entry.line_number);
        let is_match = app.search_matches.contains(&(app.scroll_offset + i));

        // Build the line
        let mut spans = Vec::new();

        // Line number
        if app.show_line_numbers {
            let line_num = format!("{:>width$} ", entry.line_number, width = line_num_width - 2);
            spans.push(Span::styled(line_num, Style::default().fg(Color::DarkGray)));
        }

        // Bookmark indicator
        if is_bookmarked {
            spans.push(Span::styled("* ", Style::default().fg(Color::Yellow)));
        }

        // Level badge
        let level_style = get_level_style(entry.level);
        spans.push(Span::styled(
            format!("{:<5} ", entry.level.label()),
            level_style,
        ));

        // Message content
        let content_width = inner.width as usize - line_num_width - 8;
        let message = if app.wrap_lines {
            entry.display().to_string()
        } else {
            let msg = entry.display();
            if msg.len() > content_width {
                format!("{}...", &msg[..content_width.saturating_sub(3)])
            } else {
                msg.to_string()
            }
        };

        // Highlight search matches in content
        if let Some(ref regex) = app.search_regex {
            let mut last_end = 0;
            for mat in regex.find_iter(&message) {
                if mat.start() > last_end {
                    spans.push(Span::raw(&message[last_end..mat.start()]));
                }
                spans.push(Span::styled(
                    &message[mat.start()..mat.end()],
                    Style::default().bg(Color::Yellow).fg(Color::Black),
                ));
                last_end = mat.end();
            }
            if last_end < message.len() {
                spans.push(Span::raw(&message[last_end..]));
            }
        } else {
            spans.push(Span::raw(message));
        }

        let line = Line::from(spans);

        // Apply selection/match highlighting
        let style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else if is_match {
            Style::default().bg(Color::Rgb(50, 50, 0))
        } else {
            Style::default()
        };

        let paragraph = Paragraph::new(line).style(style);
        frame.render_widget(paragraph, Rect::new(inner.x, y, inner.width, 1));
    }

    // Render scrollbar
    if visible.len() > viewport_height {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));

        let mut scrollbar_state = ScrollbarState::new(visible.len())
            .position(app.scroll_offset);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin { vertical: 0, horizontal: 0 }),
            &mut scrollbar_state,
        );
    }
}

fn render_input_bar(frame: &mut Frame, app: &App, area: Rect) {
    let content = match app.mode {
        Mode::Search => {
            format!("/{}█", app.search_query)
        }
        Mode::Filter => {
            format!("Filter level (e/w/i/d/t): {}█", app.filter_input)
        }
        Mode::Normal => {
            if let Some(ref msg) = app.message {
                msg.clone()
            } else {
                "Press ? for help | q to quit".to_string()
            }
        }
        Mode::Help => String::new(),
    };

    let style = match app.mode {
        Mode::Search | Mode::Filter => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::DarkGray),
    };

    let bar = Paragraph::new(content).style(style);
    frame.render_widget(bar, area);
}

fn render_help(frame: &mut Frame) {
    let area = centered_rect(60, 80, frame.area());

    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("Log Viewer Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k, ↑/↓     Move up/down"),
        Line::from("  J/K, PgUp/Dn Page up/down"),
        Line::from("  g/G          Top/bottom"),
        Line::from("  f            Toggle follow mode"),
        Line::from(""),
        Line::from(Span::styled("Search & Filter", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  /            Search (regex)"),
        Line::from("  n/N          Next/previous match"),
        Line::from("  l            Filter by level"),
        Line::from("  1-5          Quick filter (1=Error..5=Trace)"),
        Line::from("  0            Clear level filter"),
        Line::from("  c            Clear all filters"),
        Line::from(""),
        Line::from(Span::styled("Bookmarks", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  b            Toggle bookmark"),
        Line::from("  B            Clear all bookmarks"),
        Line::from("  '            Jump to next bookmark"),
        Line::from(""),
        Line::from(Span::styled("Display", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  w            Toggle line wrap"),
        Line::from("  #            Toggle line numbers"),
        Line::from(""),
        Line::from("  q, Ctrl+C    Quit"),
        Line::from("  ?            Toggle this help"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().title(" Help ").borders(Borders::ALL))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(help, area);
}

fn get_level_style(level: LogLevel) -> Style {
    match level {
        LogLevel::Error => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        LogLevel::Warn => Style::default().fg(Color::Yellow),
        LogLevel::Info => Style::default().fg(Color::Green),
        LogLevel::Debug => Style::default().fg(Color::Cyan),
        LogLevel::Trace => Style::default().fg(Color::DarkGray),
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
