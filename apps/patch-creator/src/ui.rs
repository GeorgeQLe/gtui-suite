use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
};

use crate::app::{App, HunkAction, LineType};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(if app.show_preview { 10 } else { 0 }),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_hunks(frame, app, chunks[1]);
    if app.show_preview {
        render_preview(frame, app, chunks[2]);
    }
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "PATCH CREATOR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{}/{} hunks", app.included_count(), app.hunks.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("+{}", app.total_additions()),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" "),
        Span::styled(
            format!("-{}", app.total_deletions()),
            Style::default().fg(Color::Red),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Interactive Patch Creation "));

    frame.render_widget(header, area);
}

fn render_hunks(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["", "File", "Lines", "Changes", "Status"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1);

    let visible_height = area.height.saturating_sub(3) as usize;
    let start = if app.selected_hunk >= visible_height {
        app.selected_hunk - visible_height + 1
    } else {
        0
    };

    let rows: Vec<Row> = app
        .hunks
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, hunk)| {
            let style = if i == app.selected_hunk {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let checkbox = match hunk.action {
                HunkAction::Include => "[x]",
                HunkAction::Exclude => "[ ]",
                HunkAction::Split => "[~]",
            };

            let checkbox_color = match hunk.action {
                HunkAction::Include => Color::Green,
                HunkAction::Exclude => Color::DarkGray,
                HunkAction::Split => Color::Yellow,
            };

            let additions = hunk.content.iter().filter(|l| l.line_type == LineType::Addition).count();
            let deletions = hunk.content.iter().filter(|l| l.line_type == LineType::Deletion).count();

            Row::new(vec![
                Cell::from(checkbox).style(Style::default().fg(checkbox_color)),
                Cell::from(hunk.file.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(format!("@@ -{},{} +{},{} @@", hunk.start_line, hunk.old_lines, hunk.start_line, hunk.new_lines)),
                Cell::from(format!("+{} -{}", additions, deletions)),
                Cell::from(match hunk.action {
                    HunkAction::Include => "Included",
                    HunkAction::Exclude => "Excluded",
                    HunkAction::Split => "Split",
                }).style(Style::default().fg(checkbox_color)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Percentage(30),
        Constraint::Percentage(25),
        Constraint::Length(12),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Hunks ({}/{}) ", app.selected_hunk + 1, app.hunks.len())),
        );

    frame.render_widget(table, area);
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = if let Some(hunk) = app.current_hunk() {
        hunk.content
            .iter()
            .map(|line| {
                let (prefix, color) = match line.line_type {
                    LineType::Context => (" ", Color::White),
                    LineType::Addition => ("+", Color::Green),
                    LineType::Deletion => ("-", Color::Red),
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(color)),
                    Span::styled(&line.content, Style::default().fg(color)),
                ]))
            })
            .collect()
    } else {
        vec![ListItem::new("No hunk selected")]
    };

    let title = app.current_hunk()
        .map(|h| format!(" {} ", h.file))
        .unwrap_or_else(|| " Preview ".to_string());

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
