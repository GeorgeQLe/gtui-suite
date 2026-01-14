use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Tabs},
};

use crate::app::{AlertSeverity, AlertState, App, Tab};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    render_tabs(f, app, chunks[0]);

    match app.current_tab {
        Tab::Alerts => render_alerts(f, app, chunks[1]),
        Tab::Silences => render_silences(f, app, chunks[1]),
    }

    render_status_bar(f, app, chunks[2]);

    if app.show_help {
        render_help(f);
    }

    if app.show_details {
        render_details(f, app);
    }
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["[1] Alerts", "[2] Silences"];
    let selected = match app.current_tab {
        Tab::Alerts => 0,
        Tab::Silences => 1,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Alert Manager "))
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).bold());

    f.render_widget(tabs, area);
}

fn severity_style(severity: &AlertSeverity) -> Style {
    match severity {
        AlertSeverity::Critical => Style::default().fg(Color::Red),
        AlertSeverity::Warning => Style::default().fg(Color::Yellow),
        AlertSeverity::Info => Style::default().fg(Color::Blue),
    }
}

fn state_style(state: &AlertState) -> Style {
    match state {
        AlertState::Firing => Style::default().fg(Color::Red),
        AlertState::Pending => Style::default().fg(Color::Yellow),
        AlertState::Resolved => Style::default().fg(Color::Green),
    }
}

fn render_alerts(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Alert").style(Style::default().bold()),
        Cell::from("Severity").style(Style::default().bold()),
        Cell::from("State").style(Style::default().bold()),
        Cell::from("Instance").style(Style::default().bold()),
        Cell::from("Started").style(Style::default().bold()),
    ])
    .height(1);

    let filtered = app.filtered_alerts();
    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, alert)| {
            let style = if i == app.selected_alert {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let instance = alert
                .labels
                .iter()
                .find(|(k, _)| k == "instance")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "-".to_string());

            let started = alert.starts_at.format("%H:%M:%S").to_string();

            Row::new(vec![
                Cell::from(alert.name.clone()),
                Cell::from(alert.severity.as_str()).style(severity_style(&alert.severity)),
                Cell::from(alert.state.as_str()).style(state_style(&alert.state)),
                Cell::from(instance),
                Cell::from(started),
            ])
            .style(style)
        })
        .collect();

    let filter_text = match &app.filter_severity {
        Some(s) => format!(" (filter: {}) ", s.as_str()),
        None => String::new(),
    };

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Alerts{}", filter_text)),
    );

    f.render_widget(table, area);
}

fn render_silences(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("ID").style(Style::default().bold()),
        Cell::from("Matchers").style(Style::default().bold()),
        Cell::from("Created By").style(Style::default().bold()),
        Cell::from("Comment").style(Style::default().bold()),
        Cell::from("Status").style(Style::default().bold()),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .silences
        .iter()
        .enumerate()
        .map(|(i, silence)| {
            let style = if i == app.selected_silence {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let status_style = if silence.active {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            };

            Row::new(vec![
                Cell::from(silence.id.clone()),
                Cell::from(silence.matchers.join(", ")),
                Cell::from(silence.created_by.clone()),
                Cell::from(silence.comment.clone()),
                Cell::from(if silence.active { "active" } else { "expired" }).style(status_style),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Min(25),
            Constraint::Length(12),
            Constraint::Length(30),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Silences "));

    f.render_widget(table, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let help = match app.current_tab {
        Tab::Alerts => " q:Quit | Tab:Switch | j/k:Nav | Enter:Details | s:Silence | a:Ack | f:Filter | ?:Help ",
        Tab::Silences => " q:Quit | Tab:Switch | j/k:Nav | Enter:Details | n:New | d:Delete | ?:Help ",
    };
    let paragraph = Paragraph::new(help).style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let help_text = r#"
Alert Manager - Keyboard Shortcuts

Navigation:
  j/k, Up/Down  - Navigate list
  Tab           - Switch tabs
  1/2           - Jump to tab

Alerts:
  Enter         - View details
  s             - Silence alert
  a             - Acknowledge
  f             - Cycle severity filter

Silences:
  Enter         - View details
  n             - New silence
  d             - Delete silence

General:
  ?             - Toggle help
  q             - Quit
"#;

    let paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, area);
}

fn render_details(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 70, f.area());
    f.render_widget(Clear, area);

    let content = match app.current_tab {
        Tab::Alerts => {
            if let Some(alert) = app.current_alert() {
                let labels = alert
                    .labels
                    .iter()
                    .map(|(k, v)| format!("  {}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");

                let annotations = alert
                    .annotations
                    .iter()
                    .map(|(k, v)| format!("  {}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");

                format!(
                    r#"
Alert: {}

Severity: {}
State:    {}

Labels:
{}

Annotations:
{}

Started:     {}
Fingerprint: {}
"#,
                    alert.name,
                    alert.severity.as_str(),
                    alert.state.as_str(),
                    labels,
                    annotations,
                    alert.starts_at.format("%Y-%m-%d %H:%M:%S UTC"),
                    alert.fingerprint
                )
            } else {
                "No alert selected".to_string()
            }
        }
        Tab::Silences => {
            let silence = &app.silences[app.selected_silence];
            format!(
                r#"
Silence: {}

Matchers:   {}
Created By: {}
Comment:    {}

Starts At: {}
Ends At:   {}
Status:    {}
"#,
                silence.id,
                silence.matchers.join(", "),
                silence.created_by,
                silence.comment,
                silence.starts_at.format("%Y-%m-%d %H:%M:%S UTC"),
                silence.ends_at.format("%Y-%m-%d %H:%M:%S UTC"),
                if silence.active { "Active" } else { "Expired" }
            )
        }
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Details "))
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
