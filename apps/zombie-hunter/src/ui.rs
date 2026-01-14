use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::{App, ProcessState};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_table(frame, app, chunks[1]);
    render_info(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "ZOMBIE HUNTER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} zombies", app.zombie_count()),
            Style::default().fg(Color::Red),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} orphans", app.orphan_count()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} selected", app.selected_count()),
            Style::default().fg(Color::Green),
        ),
        if app.auto_refresh {
            Span::styled(" [AUTO]", Style::default().fg(Color::Magenta))
        } else {
            Span::raw("")
        },
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Process Cleanup "));

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["", "PID", "State", "Name", "PPID", "Parent", "CPU Time", "Start"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1);

    let filtered = app.filtered_zombies();
    let visible_height = area.height.saturating_sub(3) as usize;
    let start = if app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, zombie)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let state_color = match zombie.state {
                ProcessState::Zombie => Color::Red,
                ProcessState::Defunct => Color::Yellow,
                ProcessState::Orphan => Color::Magenta,
            };

            let checkbox = if zombie.selected { "[x]" } else { "[ ]" };

            Row::new(vec![
                Cell::from(checkbox).style(Style::default().fg(
                    if zombie.selected { Color::Green } else { Color::DarkGray }
                )),
                Cell::from(zombie.pid.to_string()).style(Style::default().fg(Color::Cyan)),
                Cell::from(zombie.state.short_name()).style(Style::default().fg(state_color)),
                Cell::from(zombie.name.clone()),
                Cell::from(zombie.ppid.to_string()),
                Cell::from(zombie.parent_name.clone()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(zombie.cpu_time.clone()),
                Cell::from(zombie.start_time.clone()).style(Style::default().fg(Color::DarkGray)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Length(8),
        Constraint::Length(5),
        Constraint::Percentage(20),
        Constraint::Length(8),
        Constraint::Percentage(20),
        Constraint::Length(10),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Zombie Processes ({}/{}) ",
                    app.selected + 1, filtered.len())),
        );

    frame.render_widget(table, area);
}

fn render_info(frame: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_zombies();
    let content = if let Some(zombie) = filtered.get(app.selected) {
        Line::from(vec![
            Span::styled("Process: ", Style::default().fg(Color::Gray)),
            Span::styled(&zombie.name, Style::default().fg(Color::Cyan)),
            Span::styled(" (", Style::default().fg(Color::Gray)),
            Span::styled(zombie.state.name(), Style::default().fg(match zombie.state {
                ProcessState::Zombie => Color::Red,
                ProcessState::Defunct => Color::Yellow,
                ProcessState::Orphan => Color::Magenta,
            })),
            Span::styled(") ", Style::default().fg(Color::Gray)),
            Span::styled("Parent: ", Style::default().fg(Color::Gray)),
            Span::styled(&zombie.parent_name, Style::default().fg(Color::White)),
            Span::styled(format!(" (PID {})", zombie.ppid), Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(Span::styled("No zombie processes", Style::default().fg(Color::DarkGray)))
    };

    let info = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Details "));

    frame.render_widget(info, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
