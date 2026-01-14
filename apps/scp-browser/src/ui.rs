use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table}};
use crate::app::{App, PaneType};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(4), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("SCP BROWSER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | Connected: "),
        Span::styled(&app.connected_host, Style::default().fg(Color::Green)),
    ])).block(Block::default().borders(Borders::ALL).title(" Secure Copy Protocol "));
    frame.render_widget(header, chunks[0]);

    let pane_chunks = Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]).split(chunks[1]);

    render_file_pane(frame, pane_chunks[0], "Local", &app.local_path, &app.local_files, app.local_selected, app.active_pane == PaneType::Local);
    render_file_pane(frame, pane_chunks[1], "Remote", &app.remote_path, &app.remote_files, app.remote_selected, app.active_pane == PaneType::Remote);

    // Transfer progress
    if !app.transfers.is_empty() {
        let transfer = &app.transfers[0];
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(format!(" {} {} ({}) ", transfer.filename, transfer.direction, transfer.speed)))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(transfer.progress as u16);
        frame.render_widget(gauge, chunks[2]);
    } else {
        let empty = Paragraph::new("No active transfers")
            .block(Block::default().borders(Borders::ALL).title(" Transfers "))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, chunks[2]);
    }

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[3]);
}

fn render_file_pane(frame: &mut Frame, area: Rect, title: &str, path: &str, files: &[crate::app::FileEntry], selected: usize, active: bool) {
    let border_style = if active { Style::default().fg(Color::Cyan) } else { Style::default() };
    let rows: Vec<Row> = files.iter().enumerate().map(|(i, f)| {
        let style = if i == selected && active { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let icon = if f.is_dir { "/" } else { "" };
        let size = if f.is_dir { "-".to_string() } else { format_size(f.size) };
        Row::new(vec![
            Cell::from(format!("{}{}", f.name, icon)).style(if f.is_dir { Style::default().fg(Color::Blue) } else { Style::default() }),
            Cell::from(size),
            Cell::from(f.modified.clone()).style(Style::default().fg(Color::DarkGray)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Percentage(50), Constraint::Length(10), Constraint::Length(12)])
        .header(Row::new(["Name", "Size", "Modified"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" {} - {} ", title, path)).border_style(border_style));
    frame.render_widget(table, area);
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{}B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1}K", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0)) }
    else { format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0)) }
}
