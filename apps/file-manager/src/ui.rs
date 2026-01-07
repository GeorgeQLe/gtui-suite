use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::app::{ActivePane, App, ConfirmAction, Mode};
use crate::entry::EntryType;
use crate::pane::Pane;

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),     // Main content
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    // Split main area into panes
    let pane_constraints = if app.show_preview {
        vec![
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Percentage(30),
        ]
    } else {
        vec![
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]
    };

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(pane_constraints)
        .split(chunks[0]);

    // Render panes
    render_pane(frame, &app.left_pane, panes[0], app.active == ActivePane::Left);
    render_pane(frame, &app.right_pane, panes[1], app.active == ActivePane::Right);

    // Preview pane
    if app.show_preview && panes.len() > 2 {
        render_preview(frame, app, panes[2]);
    }

    // Status bar
    render_status_bar(frame, app, chunks[1]);

    // Render overlays
    match &app.mode {
        Mode::Search(query) => render_input_dialog(frame, "Search", query),
        Mode::Rename(name) => render_input_dialog(frame, "Rename", name),
        Mode::NewFile(name) => render_input_dialog(frame, "New File", name),
        Mode::NewDir(name) => render_input_dialog(frame, "New Directory", name),
        Mode::Confirm(action) => render_confirm_dialog(frame, action),
        Mode::Help => render_help(frame),
        Mode::Bookmarks => render_bookmarks(frame, app),
        Mode::Sort => render_sort_menu(frame),
        Mode::Normal => {}
    }
}

fn render_pane(frame: &mut Frame, pane: &Pane, area: Rect, is_active: bool) {
    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = pane.path.display().to_string();
    let title = if title.len() > area.width as usize - 4 {
        format!("...{}", &title[title.len() - (area.width as usize - 7)..])
    } else {
        title
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Calculate visible range
    let viewport_height = inner.height as usize;
    let total = pane.entries.len();

    // Adjust scroll offset to keep selection visible
    let mut scroll_offset = pane.scroll_offset;
    if pane.selected < scroll_offset {
        scroll_offset = pane.selected;
    } else if pane.selected >= scroll_offset + viewport_height {
        scroll_offset = pane.selected.saturating_sub(viewport_height) + 1;
    }

    // Render entries
    let items: Vec<ListItem> = pane.entries
        .iter()
        .skip(scroll_offset)
        .take(viewport_height)
        .enumerate()
        .map(|(i, entry)| {
            let idx = scroll_offset + i;
            let is_selected = idx == pane.selected;
            let is_marked = pane.selection.contains(&entry.path);

            // Build line content
            let mark = if is_marked { "*" } else { " " };
            let icon = entry.icon();
            let name = &entry.name;
            let size = entry.format_size();

            // Truncate name if needed
            let max_name_len = inner.width as usize - 15;
            let display_name = if name.len() > max_name_len {
                format!("{}...", &name[..max_name_len.saturating_sub(3)])
            } else {
                name.clone()
            };

            let content = format!("{} {}{:<width$} {:>7}",
                mark, icon, display_name, size,
                width = max_name_len
            );

            let style = match (is_selected, is_marked, entry.entry_type) {
                (true, _, _) => Style::default().bg(Color::Blue).fg(Color::White),
                (_, true, _) => Style::default().fg(Color::Yellow),
                (_, _, EntryType::Directory) => Style::default().fg(Color::Cyan),
                (_, _, EntryType::Symlink) => Style::default().fg(Color::Magenta),
                (_, _, EntryType::File) => Style::default(),
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);

    // Scrollbar
    if total > viewport_height {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight);

        let mut scrollbar_state = ScrollbarState::new(total)
            .position(scroll_offset);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin { vertical: 1, horizontal: 0 }),
            &mut scrollbar_state,
        );
    }
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Preview")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(entry) = app.active_pane().current_entry() {
        let preview_content = match entry.entry_type {
            EntryType::Directory => {
                // Show directory info
                let count = std::fs::read_dir(&entry.path)
                    .map(|d| d.count())
                    .unwrap_or(0);
                vec![
                    Line::from(Span::styled("Directory", Style::default().add_modifier(Modifier::BOLD))),
                    Line::from(""),
                    Line::from(format!("{} items", count)),
                    Line::from(""),
                    Line::from(entry.format_permissions()),
                ]
            }
            EntryType::File | EntryType::Symlink => {
                // Try to read file preview
                if let Ok(content) = std::fs::read_to_string(&entry.path) {
                    content.lines()
                        .take(inner.height as usize)
                        .map(|l| {
                            let s = if l.len() > inner.width as usize {
                                format!("{}...", &l[..inner.width as usize - 3])
                            } else {
                                l.to_string()
                            };
                            Line::from(s)
                        })
                        .collect()
                } else {
                    vec![
                        Line::from(Span::styled("Binary File", Style::default().add_modifier(Modifier::BOLD))),
                        Line::from(""),
                        Line::from(format!("Size: {}", entry.format_size())),
                        Line::from(""),
                        Line::from(entry.format_permissions()),
                        Line::from(""),
                        if let Some(modified) = &entry.modified {
                            Line::from(format!("Modified: {}", modified.format("%Y-%m-%d %H:%M")))
                        } else {
                            Line::from("")
                        },
                    ]
                }
            }
        };

        let preview = Paragraph::new(preview_content);
        frame.render_widget(preview, inner);
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let left_sort = format!("Sort: {}", app.left_pane.sort_method.label());
    let _right_sort = format!("Sort: {}", app.right_pane.sort_method.label());

    let message = app.message.as_deref().unwrap_or("");

    let content = format!(
        " {} | {} | {} ",
        left_sort,
        app.status_text(),
        if message.is_empty() { "? Help" } else { message }
    );

    let status = Paragraph::new(content)
        .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(status, area);
}

fn render_input_dialog(frame: &mut Frame, title: &str, input: &str) {
    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input_text = format!("{}█", input);
    let input_widget = Paragraph::new(input_text);
    frame.render_widget(input_widget, inner);
}

fn render_confirm_dialog(frame: &mut Frame, action: &ConfirmAction) {
    let area = centered_rect(50, 25, frame.area());
    frame.render_widget(Clear, area);

    let message = match action {
        ConfirmAction::Delete(paths) => {
            if paths.len() == 1 {
                format!("Delete '{}'?", paths[0].file_name().unwrap_or_default().to_string_lossy())
            } else {
                format!("Delete {} items?", paths.len())
            }
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

fn render_help(frame: &mut Frame) {
    let area = centered_rect(60, 80, frame.area());
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("File Manager Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k, ↑/↓     Move up/down"),
        Line::from("  h/l, ←/→     Parent / Enter"),
        Line::from("  g/G          Top/bottom"),
        Line::from("  Tab          Switch pane"),
        Line::from(""),
        Line::from(Span::styled("Selection", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Space        Toggle select"),
        Line::from("  *            Invert selection"),
        Line::from(""),
        Line::from(Span::styled("Operations", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  c            Copy to other pane"),
        Line::from("  m            Move to other pane"),
        Line::from("  d            Delete"),
        Line::from("  r            Rename"),
        Line::from("  n            New file"),
        Line::from("  N            New directory"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  /            Search"),
        Line::from("  s            Sort menu"),
        Line::from("  b            Bookmarks"),
        Line::from("  B            Add bookmark"),
        Line::from("  .            Toggle hidden"),
        Line::from("  p            Toggle preview"),
        Line::from("  F5           Refresh"),
        Line::from("  q            Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().title(" Help ").borders(Borders::ALL))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(help, area);
}

fn render_bookmarks(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 50, frame.area());
    frame.render_widget(Clear, area);

    let mut lines = vec![
        Line::from(Span::styled("Bookmarks", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];

    for (i, (name, path)) in app.config.bookmarks.iter().enumerate() {
        lines.push(Line::from(format!("  {} - {} ({})", i, name, path)));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Press number to go, Esc to cancel", Style::default().fg(Color::DarkGray))));

    let bookmarks = Paragraph::new(lines)
        .block(Block::default().title(" Bookmarks ").borders(Borders::ALL));

    frame.render_widget(bookmarks, area);
}

fn render_sort_menu(frame: &mut Frame) {
    let area = centered_rect(30, 30, frame.area());
    frame.render_widget(Clear, area);

    let lines = vec![
        Line::from(Span::styled("Sort By", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("  n - Name"),
        Line::from("  s - Size"),
        Line::from("  d - Date"),
        Line::from("  t - Type"),
        Line::from("  r - Reverse"),
        Line::from(""),
        Line::from(Span::styled("Esc to cancel", Style::default().fg(Color::DarkGray))),
    ];

    let menu = Paragraph::new(lines)
        .block(Block::default().title(" Sort ").borders(Borders::ALL));

    frame.render_widget(menu, area);
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
