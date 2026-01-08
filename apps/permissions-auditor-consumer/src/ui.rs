use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, View};
use crate::auditor::Severity;

pub fn render(frame: &mut Frame, app: &App) {
    match app.view {
        View::Findings => render_findings(frame, app),
        View::Detail => render_detail(frame, app),
        View::Help => render_help(frame),
    }
}

fn render_findings(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // Header
    let header = Paragraph::new(" Permissions Auditor - Consumer Edition ")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Summary bar
    let summary = app.summary();
    let summary_text = format!(
        " Total: {} | Critical: {} | High: {} | Medium: {} | Low: {} | Filter: {} ",
        summary.total,
        summary.critical,
        summary.high,
        summary.medium,
        summary.low,
        app.filter.as_str()
    );
    let summary_widget = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title(" Summary "));
    frame.render_widget(summary_widget, chunks[1]);

    // Findings list
    let findings = app.filtered_findings();
    if findings.is_empty() {
        let empty = Paragraph::new("No findings. Press 's' to start a scan.")
            .block(Block::default().borders(Borders::ALL).title(" Findings "));
        frame.render_widget(empty, chunks[2]);
    } else {
        let items: Vec<ListItem> = findings
            .iter()
            .enumerate()
            .map(|(i, finding)| {
                let style = if i == app.selected {
                    Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let severity_style = severity_color(finding.severity);
                let path = finding.path.to_string_lossy();
                let path_short = if path.len() > 50 {
                    format!("...{}", &path[path.len() - 47..])
                } else {
                    path.to_string()
                };

                let text = Line::from(vec![
                    Span::styled(
                        format!("[{:^8}] ", finding.severity.as_str()),
                        severity_style,
                    ),
                    Span::styled(
                        format!("{:<15} ", finding.finding_type.as_str()),
                        Style::default(),
                    ),
                    Span::raw(path_short),
                ]);

                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Findings "));
        frame.render_widget(list, chunks[2]);
    }

    // Status bar
    let status = render_status_bar(app);
    frame.render_widget(status, chunks[3]);
}

fn render_detail(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // Header
    let header = Paragraph::new(" Finding Details ")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Detail content
    if let Some(finding) = app.selected_finding() {
        let detail_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Min(1),
            ])
            .split(chunks[1]);

        // Info table
        let rows = vec![
            Row::new(vec![
                Cell::from("Path:"),
                Cell::from(finding.path.to_string_lossy().to_string()),
            ]),
            Row::new(vec![
                Cell::from("Type:"),
                Cell::from(finding.finding_type.as_str()),
            ]),
            Row::new(vec![
                Cell::from("Severity:"),
                Cell::from(finding.severity.as_str())
                    .style(severity_color(finding.severity)),
            ]),
            Row::new(vec![
                Cell::from("Permissions:"),
                Cell::from(finding.current_permissions.clone()),
            ]),
            Row::new(vec![
                Cell::from("Recommended:"),
                Cell::from(
                    finding.recommended_permissions.as_deref().unwrap_or("-").to_string()
                ),
            ]),
            Row::new(vec![
                Cell::from("Found:"),
                Cell::from(finding.found_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            ]),
        ];

        let widths = [Constraint::Percentage(20), Constraint::Percentage(80)];
        let table = Table::new(rows, widths)
            .block(Block::default().borders(Borders::ALL).title(" Info "));
        frame.render_widget(table, detail_chunks[0]);

        // Description and fix
        let desc_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(detail_chunks[1]);

        let description = Paragraph::new(finding.description.clone())
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, desc_chunks[0]);

        let fix = finding.fix_command.as_deref().unwrap_or("No fix command available");
        let fix_widget = Paragraph::new(fix)
            .block(Block::default().borders(Borders::ALL).title(" Fix Command "))
            .wrap(Wrap { trim: true });
        frame.render_widget(fix_widget, desc_chunks[1]);
    }

    // Status bar
    let status = Paragraph::new(" Esc: Back | f: Show fix | i: Ignore | q: Quit ")
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, chunks[2]);
}

fn render_help(frame: &mut Frame) {
    let help_text = vec![
        Line::from(Span::styled("Permissions Auditor Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j/k, arrows    Move up/down"),
        Line::from("  Enter          View details"),
        Line::from("  Esc            Go back"),
        Line::from(""),
        Line::from(Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  s              Start scan"),
        Line::from("  f              Show fix command"),
        Line::from("  i              Ignore finding"),
        Line::from("  Tab            Cycle filter"),
        Line::from("  c              Clear findings"),
        Line::from(""),
        Line::from(Span::styled("Checks Performed", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  - World-writable files and directories"),
        Line::from("  - SUID/SGID binaries"),
        Line::from("  - SSH key permissions"),
        Line::from("  - GPG key permissions"),
        Line::from(""),
        Line::from(Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  ?              Toggle help"),
        Line::from("  q              Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    frame.render_widget(help, frame.area());
}

fn render_status_bar(app: &App) -> Paragraph<'static> {
    let message = app.error.as_ref()
        .or(app.message.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("s: Scan | Tab: Filter | Enter: Details | ?: Help | q: Quit");

    let style = if app.error.is_some() {
        Style::default().bg(Color::Red).fg(Color::White)
    } else {
        Style::default().bg(Color::DarkGray)
    };

    Paragraph::new(format!(" {} ", message)).style(style)
}

fn severity_color(severity: Severity) -> Style {
    match severity {
        Severity::Critical => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        Severity::High => Style::default().fg(Color::LightRed),
        Severity::Medium => Style::default().fg(Color::Yellow),
        Severity::Low => Style::default().fg(Color::Blue),
    }
}
