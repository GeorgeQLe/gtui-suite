use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Tabs as TabsWidget},
};

use crate::app::{App, Mode};
use crate::tab::{Direction, TabLayout};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(layout::Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // Tab bar
    render_tab_bar(frame, app, chunks[0]);

    // Content area
    render_content(frame, app, chunks[1]);

    // Status bar
    render_status_bar(frame, app, chunks[2]);

    // Overlays
    if app.show_help {
        render_help(frame);
    }

    if app.mode == Mode::Launcher {
        render_launcher(frame, app);
    }
}

fn render_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let prefix = if tab.pinned { "* " } else { "" };
            let title = tab.title();
            let max_len = app.config.tabs.max_title_length;
            let truncated = if title.len() > max_len {
                format!("{}...", &title[..max_len - 3])
            } else {
                title.to_string()
            };

            let text = if app.config.tabs.show_index {
                format!("{}{}: {}", prefix, i + 1, truncated)
            } else {
                format!("{}{}", prefix, truncated)
            };

            Line::from(text)
        })
        .collect();

    let tabs = TabsWidget::new(titles)
        .select(app.active_tab)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .divider("|");

    frame.render_widget(tabs, area);
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(tab) = app.current_tab() {
        render_layout(frame, &tab.layout, area, true);
    } else {
        let empty = Paragraph::new("No tabs. Press Ctrl+T to create a new tab.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, area);
    }
}

fn render_layout(frame: &mut Frame, layout: &TabLayout, area: Rect, focused: bool) {
    match layout {
        TabLayout::Single { app_name, app_title } => {
            let border_color = if focused { Color::Cyan } else { Color::DarkGray };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(format!(" {} ", app_title));

            let inner = block.inner(area);
            frame.render_widget(block, area);

            let content = vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("  App: {}", app_name),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("  This is a placeholder for the"),
                Line::from("  application content."),
                Line::from(""),
                Line::from("  Press Ctrl+V for vertical split"),
                Line::from("  Press Ctrl+H for horizontal split"),
            ];

            let content_widget = Paragraph::new(content);
            frame.render_widget(content_widget, inner);
        }
        TabLayout::Split {
            direction,
            ratio,
            children,
            focused: focused_idx,
        } => {
            let (constraint1, constraint2) = (
                Constraint::Percentage((ratio * 100.0) as u16),
                Constraint::Percentage(((1.0 - ratio) * 100.0) as u16),
            );

            let layout_dir = match direction {
                Direction::Horizontal => layout::Direction::Horizontal,
                Direction::Vertical => layout::Direction::Vertical,
            };

            let chunks = Layout::default()
                .direction(layout_dir)
                .constraints([constraint1, constraint2])
                .split(area);

            render_layout(frame, &children.0, chunks[0], focused && *focused_idx == 0);
            render_layout(frame, &children.1, chunks[1], focused && *focused_idx == 1);
        }
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let message = app
        .message
        .as_ref()
        .cloned()
        .unwrap_or_else(|| {
            if let Some(tab) = app.current_tab() {
                format!("Tab {}/{} | {}", app.active_tab + 1, app.tabs.len(), tab.layout.focused_name())
            } else {
                "No tabs".to_string()
            }
        });

    let status = Paragraph::new(format!(" {} | Ctrl+T: New | Ctrl+Tab: Switch | ?: Help ", message))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn render_help(frame: &mut Frame) {
    let area = centered_rect(60, 80, frame.area());
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("Tabbed Shell Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Tab Management", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+t         New tab"),
        Line::from("  Ctrl+w         Close tab"),
        Line::from("  Ctrl+Tab       Next tab"),
        Line::from("  Ctrl+Shift+Tab Previous tab"),
        Line::from("  Ctrl+1-9       Go to tab N"),
        Line::from("  Ctrl+0         Go to last tab"),
        Line::from(""),
        Line::from(Span::styled("Tab Movement", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+Shift+Left  Move tab left"),
        Line::from("  Ctrl+Shift+Right Move tab right"),
        Line::from("  Ctrl+p           Pin/unpin tab"),
        Line::from(""),
        Line::from(Span::styled("Splits", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+v         Vertical split"),
        Line::from("  Ctrl+h         Horizontal split"),
        Line::from("  Ctrl+arrows    Focus pane"),
        Line::from("  Ctrl+x         Close pane"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  ?              Toggle help"),
        Line::from("  Ctrl+q         Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(help, area);
}

fn render_launcher(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 15, frame.area());
    frame.render_widget(Clear, area);

    let content = vec![
        Line::from("Enter app name:"),
        Line::from(""),
        Line::from(format!("{}|", app.launcher_input)),
    ];

    let launcher = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(" New Tab "),
        )
        .style(Style::default().bg(Color::Black));
    frame.render_widget(launcher, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup = Layout::default()
        .direction(layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}
