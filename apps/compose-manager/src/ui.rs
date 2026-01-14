use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::{App, ProjectStatus};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let running = app.projects.iter().filter(|p| p.status == ProjectStatus::Running).count();
    let header = Paragraph::new(Line::from(vec![
        Span::styled("COMPOSE MANAGER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} projects", app.projects.len()), Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(format!("{} running", running), Style::default().fg(Color::Green)),
    ])).block(Block::default().borders(Borders::ALL).title(" Docker Compose Projects "));
    frame.render_widget(header, chunks[0]);

    let mut rows: Vec<Row> = Vec::new();
    for (i, project) in app.projects.iter().enumerate() {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let (status_str, status_color) = match project.status {
            ProjectStatus::Running => ("RUNNING", Color::Green),
            ProjectStatus::Partial => ("PARTIAL", Color::Yellow),
            ProjectStatus::Stopped => ("STOPPED", Color::Red),
        };
        let expand_icon = if app.expanded == Some(i) { "v" } else { ">" };
        rows.push(Row::new(vec![
            Cell::from(expand_icon),
            Cell::from(status_str).style(Style::default().fg(status_color)),
            Cell::from(project.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{}/{}", project.running_count, project.services.len())),
            Cell::from(project.path.clone()).style(Style::default().fg(Color::DarkGray)),
        ]).style(style));

        if app.expanded == Some(i) {
            for service in &project.services {
                let svc_color = if service.status.starts_with("Up") { Color::Green } else { Color::Red };
                let ports = service.ports.join(", ");
                rows.push(Row::new(vec![
                    Cell::from("  "),
                    Cell::from(""),
                    Cell::from(format!("  {}", service.name)).style(Style::default().fg(Color::White)),
                    Cell::from(service.status.clone()).style(Style::default().fg(svc_color)),
                    Cell::from(ports).style(Style::default().fg(Color::DarkGray)),
                ]));
            }
        }
    }

    let table = Table::new(rows, [Constraint::Length(3), Constraint::Length(10), Constraint::Percentage(20), Constraint::Length(15), Constraint::Percentage(45)])
        .header(Row::new(["", "Status", "Project", "Services", "Path"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Projects ({}/{}) ", app.selected + 1, app.projects.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
