use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
};

use crate::app::{App, InputMode, View};
use crate::models::{FindingStatus, Severity};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(1)])
        .split(area);

    render_header(frame, app, chunks[0]);
    render_main(frame, app, chunks[1]);
    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let score_color = if app.stats.average_score >= 90.0 {
        Color::Green
    } else if app.stats.average_score >= 70.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "PERMISSIONS AUDITOR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("Enterprise", Style::default().fg(Color::Magenta)),
        Span::raw(" | Score: "),
        Span::styled(
            format!("{:.1}%", app.stats.average_score),
            Style::default().fg(score_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("🔴 {} Critical", app.stats.critical_findings),
            Style::default().fg(Color::Red),
        ),
        Span::raw(" "),
        Span::styled(
            format!("🟠 {} High", app.stats.high_findings),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Compliance Dashboard "));

    frame.render_widget(header, area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Dashboard => render_dashboard(frame, app, area),
        View::Systems => render_systems(frame, app, area),
        View::Compliance => render_compliance(frame, app, area),
        View::Findings => render_findings(frame, app, area),
        View::Reports => render_reports(frame, app, area),
        View::SystemDetails => render_system_details(frame, app, area),
    }
}

fn render_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let stats_lines = vec![
        format!("Total Systems: {}", app.stats.total_systems),
        format!("Systems Scanned: {}", app.stats.systems_scanned),
        format!("Average Compliance: {:.1}%", app.stats.average_score),
        String::new(),
        format!("Critical Findings: {}", app.stats.critical_findings),
        format!("High Findings: {}", app.stats.high_findings),
        String::new(),
        "Frameworks:".to_string(),
        "  🏛️ CIS Benchmarks".to_string(),
    ];

    let stats = Paragraph::new(stats_lines.join("\n"))
        .block(Block::default().borders(Borders::ALL).title(" Overview "));
    frame.render_widget(stats, chunks[0]);

    let recent: Vec<ListItem> = app
        .scans
        .iter()
        .take(5)
        .map(|scan| {
            let score_style = if scan.compliance_score >= 90.0 {
                Style::default().fg(Color::Green)
            } else if scan.compliance_score >= 70.0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Red)
            };

            ListItem::new(Line::from(vec![
                Span::raw(&scan.system_name),
                Span::raw(" - "),
                Span::styled(format!("{:.1}%", scan.compliance_score), score_style),
                Span::styled(
                    format!(" ({} passed, {} failed)", scan.checks_passed, scan.checks_failed),
                    Style::default().fg(Color::Gray),
                ),
            ]))
        })
        .collect();

    let recent_list = List::new(recent)
        .block(Block::default().borders(Borders::ALL).title(" Recent Scans "));
    frame.render_widget(recent_list, chunks[1]);
}

fn render_systems(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            if app.input_mode == InputMode::Search {
                Constraint::Length(3)
            } else {
                Constraint::Length(0)
            },
            Constraint::Min(5),
        ])
        .split(area);

    if app.input_mode == InputMode::Search {
        let search = Paragraph::new(format!("/{}", app.search_query))
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(" Search "));
        frame.render_widget(search, chunks[0]);
    }

    let filtered = app.filtered_systems();

    let header = Row::new(vec![
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Hostname").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("OS").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Score").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Last Scan").style(Style::default().add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, system)| {
            let style = if i == app.selected_system {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let status_style = match system.status {
                crate::models::SystemStatus::Online => Style::default().fg(Color::Green),
                crate::models::SystemStatus::Offline => Style::default().fg(Color::Red),
                crate::models::SystemStatus::Scanning => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::Gray),
            };

            let score = system
                .compliance_score
                .map(|s| format!("{:.1}%", s))
                .unwrap_or_else(|| "-".to_string());

            let score_style = system.compliance_score.map_or(Style::default(), |s| {
                if s >= 90.0 {
                    Style::default().fg(Color::Green)
                } else if s >= 70.0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Red)
                }
            });

            let last_scan = system
                .last_scan
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Never".to_string());

            let os = format!(
                "{} {}",
                system.os.as_deref().unwrap_or("-"),
                system.os_version.as_deref().unwrap_or("")
            );

            Row::new(vec![
                Cell::from(format!("{} {}", system.status.icon(), system.status.as_str()))
                    .style(status_style),
                Cell::from(system.name.clone()),
                Cell::from(system.hostname.clone()),
                Cell::from(os),
                Cell::from(score).style(score_style),
                Cell::from(last_scan),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(12),
            Constraint::Percentage(18),
            Constraint::Percentage(22),
            Constraint::Percentage(18),
            Constraint::Percentage(12),
            Constraint::Percentage(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Systems ({}) ", filtered.len())),
    );

    frame.render_widget(table, chunks[1]);
}

fn render_compliance(frame: &mut Frame, app: &App, area: Rect) {
    let frameworks = vec![
        ("🏛️ CIS Benchmarks", "Linux CIS Benchmarks v2.0", 85),
        ("🎖️ DISA STIG", "RHEL 8 STIG v1r10", 78),
    ];

    let items: Vec<ListItem> = frameworks
        .iter()
        .map(|(icon_name, desc, score)| {
            let score_style = if *score >= 90 {
                Style::default().fg(Color::Green)
            } else if *score >= 70 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Red)
            };

            ListItem::new(Line::from(vec![
                Span::raw(*icon_name),
                Span::raw(" - "),
                Span::styled(*desc, Style::default().fg(Color::Gray)),
                Span::raw(" | Score: "),
                Span::styled(format!("{}%", score), score_style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Compliance Frameworks "));
    frame.render_widget(list, area);
}

fn render_findings(frame: &mut Frame, app: &App, area: Rect) {
    let findings = app.all_findings();

    if findings.is_empty() {
        let empty = Paragraph::new("No findings to display")
            .block(Block::default().borders(Borders::ALL).title(" Findings "))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Sev").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("ID").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Title").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Framework").style(Style::default().add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = findings
        .iter()
        .enumerate()
        .map(|(i, finding)| {
            let style = if i == app.selected_finding {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let sev_style = match finding.check.severity {
                Severity::Critical => Style::default().fg(Color::Red),
                Severity::High => Style::default().fg(Color::Yellow),
                Severity::Medium => Style::default().fg(Color::Cyan),
                _ => Style::default().fg(Color::Gray),
            };

            let status_style = match finding.status {
                FindingStatus::Pass => Style::default().fg(Color::Green),
                FindingStatus::Fail => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::Gray),
            };

            Row::new(vec![
                Cell::from(format!("{} {}", finding.check.severity.icon(), finding.check.severity.as_str()))
                    .style(sev_style),
                Cell::from(finding.check.id.clone()),
                Cell::from(finding.check.title.clone()),
                Cell::from(format!("{} {}", finding.status.icon(), finding.status.as_str()))
                    .style(status_style),
                Cell::from(format!("{} {}", finding.check.framework.icon(), finding.check.framework.as_str())),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(40),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(" Findings ({}) ", findings.len())));

    frame.render_widget(table, area);
}

fn render_reports(frame: &mut Frame, app: &App, area: Rect) {
    let reports = vec![
        ("Executive Summary", "High-level compliance overview for leadership"),
        ("Detailed Findings", "All findings with evidence and remediation"),
        ("Trend Analysis", "Compliance score changes over time"),
        ("Remediation Playbook", "Step-by-step fix instructions"),
    ];

    let items: Vec<ListItem> = reports
        .iter()
        .map(|(name, desc)| {
            ListItem::new(Line::from(vec![
                Span::styled("📄 ", Style::default().fg(Color::Yellow)),
                Span::styled(*name, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - "),
                Span::styled(*desc, Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Report Templates "));
    frame.render_widget(list, area);
}

fn render_system_details(frame: &mut Frame, app: &App, area: Rect) {
    let Some(system) = app.systems.get(app.selected_system) else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(5)])
        .split(area);

    let score = system
        .compliance_score
        .map(|s| format!("{:.1}%", s))
        .unwrap_or_else(|| "-".to_string());

    let info_lines = vec![
        format!("Name: {}", system.name),
        format!("Hostname: {}", system.hostname),
        format!(
            "OS: {} {}",
            system.os.as_deref().unwrap_or("-"),
            system.os_version.as_deref().unwrap_or("")
        ),
        format!("Compliance Score: {}", score),
    ];

    let info = Paragraph::new(info_lines.join("\n"))
        .block(Block::default().borders(Borders::ALL).title(" System Information "));
    frame.render_widget(info, chunks[0]);

    if let Some(scan) = app.system_scan() {
        let header = Row::new(vec![
            Cell::from("Sev").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("ID").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Title").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
        ]);

        let rows: Vec<Row> = scan
            .findings
            .iter()
            .enumerate()
            .map(|(i, finding)| {
                let style = if i == app.selected_finding {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let status_style = match finding.status {
                    FindingStatus::Pass => Style::default().fg(Color::Green),
                    FindingStatus::Fail => Style::default().fg(Color::Red),
                    _ => Style::default().fg(Color::Gray),
                };

                Row::new(vec![
                    Cell::from(finding.check.severity.icon()),
                    Cell::from(finding.check.id.clone()),
                    Cell::from(finding.check.title.clone()),
                    Cell::from(format!("{} {}", finding.status.icon(), finding.status.as_str()))
                        .style(status_style),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Percentage(15),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Findings ({}) ", scan.findings.len())),
        );

        frame.render_widget(table, chunks[1]);
    } else {
        let no_scan = Paragraph::new("No scan data available. Press 's' to scan.")
            .block(Block::default().borders(Borders::ALL).title(" Findings "))
            .alignment(Alignment::Center);
        frame.render_widget(no_scan, chunks[1]);
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = app.status_text();
    let style = Style::default().bg(Color::DarkGray);
    let paragraph = Paragraph::new(format!(" {} ", status)).style(style);
    frame.render_widget(paragraph, area);
}
