use ratatui::{prelude::*, widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table}};
use crate::app::{App, FanMode};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(frame.area());

    let temp_color = if app.cpu_temp > 70.0 { Color::Red } else if app.cpu_temp > 55.0 { Color::Yellow } else { Color::Green };
    let header = Paragraph::new(Line::from(vec![
        Span::styled("FAN CONTROL", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | CPU Temp: "),
        Span::styled(format!("{:.1}C", app.cpu_temp), Style::default().fg(temp_color).add_modifier(Modifier::BOLD)),
    ])).block(Block::default().borders(Borders::ALL).title(" /sys/class/hwmon "));
    frame.render_widget(header, chunks[0]);

    let fan_area = chunks[1];
    let fan_chunks = Layout::default().direction(Direction::Vertical)
        .constraints(app.fans.iter().map(|_| Constraint::Length(3)).collect::<Vec<_>>())
        .split(fan_area);

    for (i, fan) in app.fans.iter().enumerate() {
        if i >= fan_chunks.len() { break; }
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let mode_str = match fan.mode { FanMode::Auto => "AUTO", FanMode::Manual => "MAN" };
        let color = if fan.pwm > 70 { Color::Red } else if fan.pwm > 40 { Color::Yellow } else { Color::Green };
        let label = format!("{} [{}] {} RPM ({}%)", fan.name, mode_str, fan.rpm, fan.pwm);
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).style(style))
            .gauge_style(Style::default().fg(color))
            .percent(fan.pwm as u16)
            .label(label);
        frame.render_widget(gauge, fan_chunks[i]);
    }

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[2]);
}
