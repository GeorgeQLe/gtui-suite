use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs}};
use crate::app::{App, SensorType};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("SENSOR DASHBOARD", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} chips detected", app.chips.len()), Style::default().fg(Color::Yellow)),
    ])).block(Block::default().borders(Borders::ALL).title(" lm-sensors "));
    frame.render_widget(header, chunks[0]);

    let chip_names: Vec<Line> = app.chips.iter().map(|c| Line::from(c.name.clone())).collect();
    let tabs = Tabs::new(chip_names)
        .block(Block::default().borders(Borders::ALL).title(" Sensor Chips "))
        .select(app.selected_chip)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, chunks[1]);

    if let Some(chip) = app.chips.get(app.selected_chip) {
        let rows: Vec<Row> = chip.sensors.iter().enumerate().map(|(i, s)| {
            let style = if i == app.selected_sensor { Style::default().bg(Color::DarkGray) } else { Style::default() };
            let type_str = match s.sensor_type {
                SensorType::Temperature => "TEMP",
                SensorType::Voltage => "VOLT",
                SensorType::Fan => "FAN",
                SensorType::Power => "PWR",
                SensorType::Current => "AMP",
            };
            let value_color = match s.sensor_type {
                SensorType::Temperature => {
                    if let Some(crit) = s.critical {
                        if s.value > crit * 0.9 { Color::Red }
                        else if s.value > crit * 0.7 { Color::Yellow }
                        else { Color::Green }
                    } else { Color::Green }
                },
                _ => Color::Cyan,
            };
            let critical_str = s.critical.map(|c| format!("{:.1}", c)).unwrap_or_default();
            Row::new(vec![
                Cell::from(type_str).style(Style::default().fg(Color::Magenta)),
                Cell::from(s.name.clone()),
                Cell::from(format!("{:.1} {}", s.value, s.unit)).style(Style::default().fg(value_color)),
                Cell::from(format!("{:.1}", s.min)).style(Style::default().fg(Color::DarkGray)),
                Cell::from(format!("{:.1}", s.max)).style(Style::default().fg(Color::DarkGray)),
                Cell::from(critical_str).style(Style::default().fg(Color::Red)),
            ]).style(style)
        }).collect();

        let table = Table::new(rows, [Constraint::Length(6), Constraint::Percentage(30), Constraint::Length(15), Constraint::Length(8), Constraint::Length(8), Constraint::Length(8)])
            .header(Row::new(["Type", "Name", "Value", "Min", "Max", "Crit"]).style(Style::default().fg(Color::Yellow)))
            .block(Block::default().borders(Borders::ALL).title(format!(" {} - {} ", chip.name, chip.adapter)));
        frame.render_widget(table, chunks[2]);
    }

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[3]);
}
