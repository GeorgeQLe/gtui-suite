use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_table(frame, app, chunks[1]);
    render_details(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let over = app.over_quota_count();

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "QUOTA MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} users", app.quotas.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} over quota", over),
            Style::default().fg(if over > 0 { Color::Red } else { Color::Green }),
        ),
        Span::raw(" | "),
        Span::styled(
            if app.show_groups { "Groups" } else { "Users" },
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Disk Quota Management "));

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["User", "FS", "Blocks Used", "Soft", "Hard", "%", "Grace"]
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
        .quotas
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, quota)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let pct = quota.block_percent();
            let pct_color = if quota.is_over_hard() {
                Color::Red
            } else if quota.is_over_soft() {
                Color::Yellow
            } else {
                Color::Green
            };

            let grace = quota.grace_time.clone().unwrap_or_else(|| "-".to_string());

            Row::new(vec![
                Cell::from(quota.user.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(quota.filesystem.clone()),
                Cell::from(format_blocks(quota.blocks_used)),
                Cell::from(format_blocks(quota.blocks_soft)),
                Cell::from(format_blocks(quota.blocks_hard)),
                Cell::from(format!("{:.0}%", pct)).style(Style::default().fg(pct_color)),
                Cell::from(grace).style(Style::default().fg(
                    if quota.grace_time.is_some() { Color::Yellow } else { Color::DarkGray }
                )),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(6),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Quotas ({}/{}) ", app.selected + 1, app.quotas.len())),
        );

    frame.render_widget(table, area);
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(quota) = app.quotas.get(app.selected) {
        let pct = quota.block_percent();
        let color = if quota.is_over_hard() {
            Color::Red
        } else if quota.is_over_soft() {
            Color::Yellow
        } else {
            Color::Green
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(format!(" {} - Disk Usage ", quota.user)))
            .gauge_style(Style::default().fg(color))
            .ratio((pct / 100.0).min(1.0))
            .label(format!("{} / {} ({:.1}%)",
                format_blocks(quota.blocks_used),
                format_blocks(quota.blocks_hard),
                pct));

        frame.render_widget(gauge, area);
    } else {
        let empty = Block::default().borders(Borders::ALL).title(" Details ");
        frame.render_widget(empty, area);
    }
}

fn format_blocks(blocks: u64) -> String {
    let bytes = blocks * 1024;
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
