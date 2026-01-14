use ratatui::{prelude::*, widgets::{Block, Borders, Gauge, Paragraph, Sparkline}};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(5), Constraint::Length(1)]).split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled("GPU MONITOR", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(format!("{} GPUs detected", app.gpus.len()), Style::default().fg(Color::Yellow)),
    ])).block(Block::default().borders(Borders::ALL).title(" Graphics Processing Units "));
    frame.render_widget(header, chunks[0]);

    let gpu_area = chunks[1];
    let gpu_chunks = Layout::default().direction(Direction::Vertical)
        .constraints(app.gpus.iter().map(|_| Constraint::Length(5)).collect::<Vec<_>>())
        .split(gpu_area);

    for (i, gpu) in app.gpus.iter().enumerate() {
        if i >= gpu_chunks.len() { break; }
        let style = if i == app.selected { Style::default().bg(Color::DarkGray) } else { Style::default() };

        let inner_chunks = Layout::default().direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Length(1), Constraint::Length(1)])
            .split(gpu_chunks[i]);

        let info = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(&gpu.name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(format!(" | {}C | {:.0}W | {} MHz", gpu.temperature as u32, gpu.power_draw, gpu.clock_speed)),
            ]),
        ]).block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT).title(format!(" GPU {} ", i)).style(style));
        frame.render_widget(info, inner_chunks[0]);

        let util_color = if gpu.utilization > 80 { Color::Red } else if gpu.utilization > 50 { Color::Yellow } else { Color::Green };
        let util_gauge = Gauge::default()
            .gauge_style(Style::default().fg(util_color))
            .percent(gpu.utilization as u16)
            .label(format!("GPU: {}%", gpu.utilization));
        frame.render_widget(util_gauge, inner_chunks[1]);

        let mem_percent = ((gpu.memory_used as f32 / gpu.memory_total as f32) * 100.0) as u16;
        let mem_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Magenta))
            .percent(mem_percent)
            .label(format!("MEM: {} / {} MB", gpu.memory_used, gpu.memory_total));
        frame.render_widget(mem_gauge, inner_chunks[2]);
    }

    let history_data: Vec<u64> = app.utilization_history.iter().map(|&v| v as u64).collect();
    let sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" GPU 0 Utilization History "))
        .data(&history_data)
        .style(Style::default().fg(Color::Green));
    frame.render_widget(sparkline, chunks[2]);

    frame.render_widget(Paragraph::new(format!(" {} ", app.status_text())).style(Style::default().bg(Color::DarkGray)), chunks[3]);
}
