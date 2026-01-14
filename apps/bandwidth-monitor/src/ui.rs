use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
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
    render_table(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let pause_indicator = if app.paused { " [PAUSED]" } else { "" };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "BANDWIDTH MONITOR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("↓ ", Style::default().fg(Color::Green)),
        Span::raw(format_rate(app.total_download)),
        Span::raw(" "),
        Span::styled("↑ ", Style::default().fg(Color::Red)),
        Span::raw(format_rate(app.total_upload)),
        Span::styled(pause_indicator, Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Network Usage "));

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["PID", "Process", "↓ Rate", "↑ Rate", "↓ Total", "↑ Total"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        });

    let header = Row::new(header_cells).bottom_margin(1);

    let rows: Vec<Row> = app
        .processes
        .iter()
        .enumerate()
        .map(|(i, proc)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let cells = vec![
                Cell::from(proc.pid.to_string()),
                Cell::from(proc.name.clone()).style(Style::default().fg(Color::White)),
                Cell::from(proc.download_rate_formatted()).style(Style::default().fg(Color::Green)),
                Cell::from(proc.upload_rate_formatted()).style(Style::default().fg(Color::Red)),
                Cell::from(proc.total_download_formatted()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(proc.total_upload_formatted()).style(Style::default().fg(Color::DarkGray)),
            ];

            Row::new(cells).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Length(15),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Processes ({}) ",
            app.processes.len()
        )));

    frame.render_widget(table, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn format_rate(bytes_per_sec: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec as f64 / MB as f64)
    } else if bytes_per_sec >= KB {
        format!("{:.1} KB/s", bytes_per_sec as f64 / KB as f64)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}
