use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{App, FileType};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(if app.show_details { 8 } else { 0 }),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_table(frame, app, chunks[1]);
    if app.show_details {
        render_details(frame, app, chunks[2]);
    }
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "INODE INSPECTOR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} inodes", app.inodes.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} blocks", app.total_blocks()),
            Style::default().fg(Color::Green),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Filesystem Inode Analysis "));

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Inode", "Type", "Path", "Size", "Links", "Mode"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1);

    let visible_height = area.height.saturating_sub(3) as usize;
    let start = if app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    let rows: Vec<Row> = app
        .inodes
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, inode)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let type_color = match inode.file_type {
                FileType::Regular => Color::White,
                FileType::Directory => Color::Blue,
                FileType::Symlink => Color::Cyan,
                FileType::BlockDevice => Color::Yellow,
                FileType::CharDevice => Color::Yellow,
                FileType::Socket => Color::Magenta,
                FileType::Fifo => Color::Green,
            };

            Row::new(vec![
                Cell::from(inode.inode.to_string()).style(Style::default().fg(Color::Cyan)),
                Cell::from(format!("{}", inode.file_type.code())).style(Style::default().fg(type_color)),
                Cell::from(truncate(&inode.path, 35)),
                Cell::from(format_size(inode.size)),
                Cell::from(inode.links.to_string()),
                Cell::from(inode.mode.clone()).style(Style::default().fg(Color::DarkGray)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(12),
        Constraint::Length(4),
        Constraint::Percentage(40),
        Constraint::Length(10),
        Constraint::Length(6),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Inodes ({}/{}) ", app.selected + 1, app.inodes.len())),
        );

    frame.render_widget(table, area);
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(inode) = app.current_inode() {
        vec![
            Line::from(vec![
                Span::styled("Inode: ", Style::default().fg(Color::Gray)),
                Span::styled(inode.inode.to_string(), Style::default().fg(Color::Cyan)),
                Span::styled("  Device: ", Style::default().fg(Color::Gray)),
                Span::styled(&inode.device, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::Gray)),
                Span::styled(inode.file_type.name(), Style::default().fg(Color::White)),
                Span::styled("  Blocks: ", Style::default().fg(Color::Gray)),
                Span::styled(inode.blocks.to_string(), Style::default().fg(Color::Green)),
                Span::styled("  Block Size: ", Style::default().fg(Color::Gray)),
                Span::styled(inode.block_size.to_string(), Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("UID: ", Style::default().fg(Color::Gray)),
                Span::styled(inode.uid.to_string(), Style::default().fg(Color::Yellow)),
                Span::styled("  GID: ", Style::default().fg(Color::Gray)),
                Span::styled(inode.gid.to_string(), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Access: ", Style::default().fg(Color::Gray)),
                Span::styled(&inode.atime, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Modify: ", Style::default().fg(Color::Gray)),
                Span::styled(&inode.mtime, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Change: ", Style::default().fg(Color::Gray)),
                Span::styled(&inode.ctime, Style::default().fg(Color::White)),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled("No inode selected", Style::default().fg(Color::DarkGray)))]
    };

    let details = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Details "));

    frame.render_widget(details, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max-3])
    } else {
        s.to_string()
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1}G", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.1}M", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1}K", bytes as f64 / 1_000.0)
    } else {
        format!("{}B", bytes)
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
