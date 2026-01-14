use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
};

use crate::app::{App, CharRange, ViewMode};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_tabs(frame, app, chunks[1]);

    match app.mode {
        ViewMode::Table => render_table(frame, app, chunks[2]),
        ViewMode::Detail => render_detail(frame, app, chunks[2]),
    }

    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "ASCII TABLE",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} characters", app.chars.len()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            app.range.name(),
            Style::default().fg(Color::Magenta),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Character Reference "));

    frame.render_widget(header, area);
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Control (1)", "Printable (2)", "Extended (3)", "All (4)"];
    let selected = match app.range {
        CharRange::Control => 0,
        CharRange::Printable => 1,
        CharRange::Extended => 2,
        CharRange::All => 3,
    };

    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title(" Range "));

    frame.render_widget(tabs, area);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Dec", "Hex", "Oct", "Bin", "Char", "Name"]
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
        .chars
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_height)
        .map(|(i, c)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let char_style = if c.code < 32 || c.code == 127 {
                Style::default().fg(Color::Red)
            } else if c.code >= 127 {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default().fg(Color::Green)
            };

            Row::new(vec![
                Cell::from(format!("{:>3}", c.code)),
                Cell::from(c.hex.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(c.octal.clone()),
                Cell::from(c.binary.clone()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(c.char_display.clone()).style(char_style),
                Cell::from(c.name.clone()),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(6),
        Constraint::Min(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" ASCII Characters ({}/{}) ", app.selected + 1, app.chars.len())),
        );

    frame.render_widget(table, area);
}

fn render_detail(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(c) = app.selected_char() {
        let detail_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Character: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &c.char_display,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Name: ", Style::default().fg(Color::Gray)),
                Span::styled(&c.name, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Decimal: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    c.code.to_string(),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Hexadecimal: ", Style::default().fg(Color::Gray)),
                Span::styled(&c.hex, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("  Octal: ", Style::default().fg(Color::Gray)),
                Span::styled(&c.octal, Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::styled("  Binary: ", Style::default().fg(Color::Gray)),
                Span::styled(&c.binary, Style::default().fg(Color::Blue)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Category: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    if c.code < 32 {
                        "Control Character"
                    } else if c.code == 32 {
                        "Whitespace"
                    } else if c.code < 127 {
                        "Printable"
                    } else if c.code == 127 {
                        "Delete"
                    } else {
                        "Extended ASCII"
                    },
                    Style::default().fg(Color::White),
                ),
            ]),
        ];

        let detail = Paragraph::new(detail_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Character Detail (Enter to go back) "),
        );

        frame.render_widget(detail, area);
    } else {
        let placeholder = Paragraph::new("No character selected")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Detail "));
        frame.render_widget(placeholder, area);
    }
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
