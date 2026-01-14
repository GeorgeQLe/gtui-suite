use ratatui::{
    prelude::*,
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Cell, Paragraph, Row, Sparkline, Table},
};

use crate::app::{App, MemoryCategory, ViewMode};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);

    match app.view_mode {
        ViewMode::Overview => render_overview(frame, app, chunks[1]),
        ViewMode::Regions => render_regions(frame, app, chunks[1]),
        ViewMode::Allocations => render_allocations(frame, app, chunks[1]),
        ViewMode::Timeline => render_timeline(frame, app, chunks[1]),
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let rec_indicator = if app.is_recording {
        Span::styled(" [REC] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("")
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "MEMORY PROFILER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        rec_indicator,
        Span::raw(" | "),
        Span::styled(
            format!("Heap: {:.1}MB", app.current_heap_mb()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("RSS: {:.1}MB", app.current_rss_mb()),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Total: {}", format_size(app.total_memory())),
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Memory Analysis "));

    frame.render_widget(header, area);
}

fn render_overview(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(5),
        ])
        .split(area);

    // Memory breakdown bar chart
    let categories = [
        (MemoryCategory::Heap, Color::Red),
        (MemoryCategory::Stack, Color::Green),
        (MemoryCategory::Code, Color::Blue),
        (MemoryCategory::SharedLib, Color::Yellow),
        (MemoryCategory::Anonymous, Color::Magenta),
        (MemoryCategory::MappedFile, Color::Cyan),
    ];

    let bars: Vec<Bar> = categories
        .iter()
        .map(|(cat, color)| {
            let size: u64 = app.regions
                .iter()
                .filter(|r| r.category == *cat)
                .map(|r| r.size_bytes)
                .sum();
            let mb = size as f64 / 1_000_000.0;
            Bar::default()
                .value(mb as u64)
                .label(Line::from(cat.name()))
                .style(Style::default().fg(*color))
        })
        .collect();

    let chart = BarChart::default()
        .block(Block::default().borders(Borders::ALL).title(" Memory Breakdown (MB) "))
        .data(BarGroup::default().bars(&bars))
        .bar_width(10)
        .bar_gap(2)
        .max(200);

    frame.render_widget(chart, chunks[0]);

    // Memory timeline sparkline
    let data: Vec<u64> = app.samples.iter().map(|s| s.heap_mb as u64).collect();
    let sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" Heap Timeline "))
        .data(&data)
        .max(200)
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(sparkline, chunks[1]);
}

fn render_regions(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Region", "Category", "Size", "RSS", "Shared", "Private", "Perm"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1);

    let visible_height = area.height.saturating_sub(3) as usize;
    let start = if app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    let rows: Vec<Row> = app
        .regions
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, region)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let cat_color = match region.category {
                MemoryCategory::Heap => Color::Red,
                MemoryCategory::Stack => Color::Green,
                MemoryCategory::Code => Color::Blue,
                MemoryCategory::SharedLib => Color::Yellow,
                MemoryCategory::MappedFile => Color::Cyan,
                MemoryCategory::Anonymous => Color::Magenta,
                MemoryCategory::Data => Color::White,
            };

            Row::new(vec![
                Cell::from(region.name.clone()).style(Style::default().fg(Color::White)),
                Cell::from(region.category.name()).style(Style::default().fg(cat_color)),
                Cell::from(format_size(region.size_bytes)),
                Cell::from(format_size(region.resident_bytes)),
                Cell::from(format_size(region.shared_bytes)),
                Cell::from(format_size(region.private_bytes)),
                Cell::from(region.permission.clone()).style(Style::default().fg(Color::DarkGray)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Percentage(25),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(6),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Memory Regions ({}/{}) ", app.selected + 1, app.regions.len())),
        );

    frame.render_widget(table, area);
}

fn render_allocations(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Function", "File", "Allocs", "Total", "Live"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1);

    let visible_height = area.height.saturating_sub(3) as usize;
    let start = if app.selected >= visible_height {
        app.selected - visible_height + 1
    } else {
        0
    };

    let rows: Vec<Row> = app
        .allocations
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, alloc)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let live_pct = if alloc.total_bytes > 0 {
                (alloc.live_bytes as f64 / alloc.total_bytes as f64) * 100.0
            } else {
                0.0
            };

            let live_color = if live_pct > 80.0 {
                Color::Red
            } else if live_pct > 50.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            Row::new(vec![
                Cell::from(alloc.function.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(format!("{}:{}", alloc.file, alloc.line)).style(Style::default().fg(Color::DarkGray)),
                Cell::from(format_count(alloc.alloc_count)),
                Cell::from(format_size(alloc.total_bytes)),
                Cell::from(format_size(alloc.live_bytes)).style(Style::default().fg(live_color)),
            ]).style(style)
        })
        .collect();

    let widths = [
        Constraint::Percentage(25),
        Constraint::Percentage(30),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Allocation Sites ({}/{}) ", app.selected + 1, app.allocations.len())),
        );

    frame.render_widget(table, area);
}

fn render_timeline(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Heap timeline
    let heap_data: Vec<u64> = app.samples.iter().map(|s| s.heap_mb as u64).collect();
    let heap_sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" Heap Memory (MB) "))
        .data(&heap_data)
        .max(200)
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(heap_sparkline, chunks[0]);

    // RSS timeline
    let rss_data: Vec<u64> = app.samples.iter().map(|s| s.rss_mb as u64).collect();
    let rss_sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" RSS Memory (MB) "))
        .data(&rss_data)
        .max(250)
        .style(Style::default().fg(Color::Green));

    frame.render_widget(rss_sparkline, chunks[1]);
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1}G", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.1}M", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1}K", bytes as f64 / 1_000.0)
    } else {
        format!("{}B", bytes)
    }
}

fn format_count(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
