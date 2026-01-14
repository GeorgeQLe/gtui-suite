use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
};

use crate::app::{App, ColorFormat};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(12),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_format_tabs(frame, app, chunks[1]);
    render_input(frame, app, chunks[2]);
    render_conversions(frame, app, chunks[3]);
    render_saved(frame, app, chunks[4]);
    render_status(frame, app, chunks[5]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "COLOR CONVERTER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} saved colors", app.saved.len()),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Converter "));

    frame.render_widget(header, area);
}

fn render_format_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<&str> = ColorFormat::all()
        .iter()
        .map(|f| f.name())
        .collect();

    let selected = match app.format {
        ColorFormat::Hex => 0,
        ColorFormat::Rgb => 1,
        ColorFormat::Hsl => 2,
        ColorFormat::Hsv => 3,
        ColorFormat::Cmyk => 4,
    };

    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title(" Input Format (Tab to switch) "));

    frame.render_widget(tabs, area);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let placeholder = match app.format {
        ColorFormat::Hex => "Enter hex (e.g., FF5733)",
        ColorFormat::Rgb => "Enter RGB (e.g., 255, 87, 51)",
        ColorFormat::Hsl => "Enter HSL (e.g., 11, 100, 60)",
        ColorFormat::Hsv => "Enter HSV (e.g., 11, 80, 100)",
        ColorFormat::Cmyk => "Enter CMYK (e.g., 0, 66, 80, 0)",
    };

    let display = if app.input.is_empty() {
        placeholder.to_string()
    } else {
        format!("{}_ ", app.input)
    };

    let style = if app.input.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    // Add color preview if we have a valid color
    let mut lines = vec![Line::from(Span::styled(display, style))];

    if let Some(ref color) = app.current {
        let (r, g, b) = color.rgb;
        lines.push(Line::from(vec![
            Span::raw("  Preview: "),
            Span::styled(
                "████████",
                Style::default().fg(Color::Rgb(r, g, b)),
            ),
        ]));
    }

    let input = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} Input ", app.format.name()))
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(input, area);
}

fn render_conversions(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref color) = app.current {
        let conversions = vec![
            Line::from(vec![
                Span::styled("    HEX: ", Style::default().fg(Color::Gray)),
                Span::styled(&color.hex, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("    RGB: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("rgb({}, {}, {})", color.rgb.0, color.rgb.1, color.rgb.2),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("    HSL: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!(
                        "hsl({:.0}°, {:.1}%, {:.1}%)",
                        color.hsl.0, color.hsl.1, color.hsl.2
                    ),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled("    HSV: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!(
                        "hsv({:.0}°, {:.1}%, {:.1}%)",
                        color.hsv.0, color.hsv.1, color.hsv.2
                    ),
                    Style::default().fg(Color::Magenta),
                ),
            ]),
            Line::from(vec![
                Span::styled("   CMYK: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!(
                        "cmyk({:.1}%, {:.1}%, {:.1}%, {:.1}%)",
                        color.cmyk.0, color.cmyk.1, color.cmyk.2, color.cmyk.3
                    ),
                    Style::default().fg(Color::Blue),
                ),
            ]),
        ];

        let conv = Paragraph::new(conversions)
            .block(Block::default().borders(Borders::ALL).title(" Conversions "));

        frame.render_widget(conv, area);
    } else {
        let placeholder = Paragraph::new("Enter a valid color value to see conversions")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Conversions "));
        frame.render_widget(placeholder, area);
    }
}

fn render_saved(frame: &mut Frame, app: &App, area: Rect) {
    if app.saved.is_empty() {
        let placeholder = Paragraph::new("No saved colors. Press Ctrl+S to save current color.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Saved Colors "));
        frame.render_widget(placeholder, area);
        return;
    }

    let items: Vec<ListItem> = app
        .saved
        .iter()
        .enumerate()
        .map(|(i, saved)| {
            let style = if i == app.selected_saved {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let (r, g, b) = saved.values.rgb;
            let line = Line::from(vec![
                Span::styled("██ ", Style::default().fg(Color::Rgb(r, g, b))),
                Span::styled(&saved.name, Style::default().fg(Color::White)),
                Span::raw(" - "),
                Span::styled(&saved.values.hex, Style::default().fg(Color::Cyan)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Saved Colors ({}) ", app.saved.len())),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
