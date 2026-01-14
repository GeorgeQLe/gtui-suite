use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table}};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let visible = app.visible_users();
    let header = Paragraph::new(Line::from(vec![
        Span::styled("USER MANAGER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} users", visible.len()), Style::default().fg(Color::Yellow)),
        Span::raw(if app.show_system_users { " (all)" } else { " (normal)" }),
    ])).block(Block::default().borders(Borders::ALL).title(" /etc/passwd "));
    frame.render_widget(header, chunks[0]);

    let rows: Vec<Row> = visible.iter().enumerate().map(|(i, u)| {
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        Row::new(vec![
            Cell::from(u.username.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{}", u.uid)),
            Cell::from(u.home.clone()),
            Cell::from(u.shell.clone()).style(Style::default().fg(Color::Yellow)),
            Cell::from(u.groups.join(", ")).style(Style::default().fg(Color::DarkGray)),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Length(15), Constraint::Length(8), Constraint::Percentage(25), Constraint::Percentage(20), Constraint::Percentage(30)])
        .header(Row::new(["Username", "UID", "Home", "Shell", "Groups"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(format!(" Users ({}/{}) ", app.selected + 1, visible.len())));
    frame.render_widget(table, chunks[1]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
