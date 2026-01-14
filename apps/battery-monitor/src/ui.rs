use ratatui::{prelude::*, widgets::{Block, Borders, Gauge, Paragraph, Sparkline}};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(6), Constraint::Length(5), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("BATTERY MONITOR", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} batteries", app.batteries.len()), Style::default().fg(Color::Yellow)),
    ])).block(Block::default().borders(Borders::ALL).title(" /sys/class/power_supply "));
    frame.render_widget(header, chunks[0]);

    let bat_chunks = Layout::default().direction(Direction::Vertical)
        .constraints(app.batteries.iter().map(|_| Constraint::Length(3)).collect::<Vec<_>>())
        .split(chunks[1]);

    for (i, bat) in app.batteries.iter().enumerate() {
        if i >= bat_chunks.len() { break; }
        let color = match bat.percentage {
            0..=20 => Color::Red,
            21..=50 => Color::Yellow,
            _ => Color::Green,
        };
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };
        let label = format!("{}: {}% ({}) - {:.1}V {:.1}W {}",
            bat.name, bat.percentage, bat.status, bat.voltage, bat.power_draw.abs(),
            bat.time_remaining.as_deref().unwrap_or(""));
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(bat.name.clone()).style(style))
            .gauge_style(Style::default().fg(color))
            .percent(bat.percentage as u16)
            .label(label);
        frame.render_widget(gauge, bat_chunks[i]);
    }

    let history_data: Vec<u64> = app.history.iter().map(|&v| v as u64).collect();
    let sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" Battery History (BAT0) "))
        .data(&history_data)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(sparkline, chunks[2]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[3]);
}
