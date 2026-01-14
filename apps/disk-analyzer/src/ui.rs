use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

use crate::app::App;

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
    render_tree(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "DISK ANALYZER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            app.current_path.to_string_lossy().to_string(),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Directory "));

    frame.render_widget(header, area);
}

fn render_tree(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(30)])
        .split(area);

    render_file_list(frame, app, chunks[0]);
    render_size_bars(frame, app, chunks[1]);
}

fn render_file_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .flat_list
        .iter()
        .enumerate()
        .map(|(i, flat_entry)| {
            let entry = &flat_entry.entry;
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let indent = "  ".repeat(flat_entry.depth);
            let icon = if entry.is_dir {
                if entry.expanded {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };

            let type_icon = if entry.is_dir { "📁" } else { "📄" };

            let line = Line::from(vec![
                Span::raw(indent),
                Span::styled(icon, Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{} ", type_icon)),
                Span::styled(&entry.name, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    entry.size_formatted(),
                    Style::default().fg(Color::Cyan),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Files "));

    frame.render_widget(list, area);
}

fn render_size_bars(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Size Distribution ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.flat_list.is_empty() || app.total_size == 0 {
        return;
    }

    // Get top entries by size
    let mut entries: Vec<_> = app
        .flat_list
        .iter()
        .filter(|fe| fe.depth == 1)
        .collect();
    entries.sort_by(|a, b| b.entry.size.cmp(&a.entry.size));
    entries.truncate(inner.height as usize);

    let constraints: Vec<Constraint> = entries
        .iter()
        .map(|_| Constraint::Length(1))
        .collect();

    if constraints.is_empty() {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, fe) in entries.iter().enumerate() {
        if i >= chunks.len() {
            break;
        }

        let percentage = fe.entry.percentage(app.total_size);
        let label = format!(
            "{}: {:.1}%",
            truncate_name(&fe.entry.name, 12),
            percentage
        );

        let color = if fe.entry.is_dir {
            Color::Blue
        } else {
            Color::Green
        };

        let gauge = Gauge::default()
            .ratio(percentage / 100.0)
            .label(label)
            .gauge_style(Style::default().fg(color));

        frame.render_widget(gauge, chunks[i]);
    }
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len - 3])
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
