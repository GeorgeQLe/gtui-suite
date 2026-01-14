use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Percentage(40),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_stats(frame, app, chunks[1]);
    render_text(frame, app, chunks[2]);
    render_frequency(frame, app, chunks[3]);
    render_status(frame, app, chunks[4]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "TEXT STATS",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} words", app.stats.words),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} chars", app.stats.characters),
            Style::default().fg(Color::Green),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Analyzer "));

    frame.render_widget(header, area);
}

fn render_stats(frame: &mut Frame, app: &App, area: Rect) {
    let stats = &app.stats;

    let stats_text = vec![
        Line::from(vec![
            Span::styled("  Characters: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", stats.characters), Style::default().fg(Color::Cyan)),
            Span::raw("  (no spaces: "),
            Span::styled(format!("{}", stats.characters_no_spaces), Style::default().fg(Color::Cyan)),
            Span::raw(")"),
        ]),
        Line::from(vec![
            Span::styled("  Words: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", stats.words), Style::default().fg(Color::Green)),
            Span::styled("     Sentences: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", stats.sentences), Style::default().fg(Color::Green)),
            Span::styled("     Paragraphs: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", stats.paragraphs), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("  Avg word length: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.1}", stats.avg_word_length), Style::default().fg(Color::Yellow)),
            Span::styled("     Avg sentence length: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.1} words", stats.avg_sentence_length), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  Reading time: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1} min", stats.reading_time_mins),
                Style::default().fg(Color::Magenta),
            ),
            Span::styled("     Speaking time: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1} min", stats.speaking_time_mins),
                Style::default().fg(Color::Magenta),
            ),
        ]),
    ];

    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title(" Statistics "));

    frame.render_widget(stats_widget, area);
}

fn render_text(frame: &mut Frame, app: &App, area: Rect) {
    let display = if app.text.is_empty() {
        "Start typing to analyze text...".to_string()
    } else {
        format!("{}_", app.text)
    };

    let style = if app.text.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let text = Paragraph::new(display)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Text Input ")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(text, area);
}

fn render_frequency(frame: &mut Frame, app: &App, area: Rect) {
    if app.stats.word_frequency.is_empty() {
        let placeholder = Paragraph::new("Word frequency will appear here")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Word Frequency "));
        frame.render_widget(placeholder, area);
        return;
    }

    let max_count = app.stats.word_frequency.first().map(|(_, c)| *c).unwrap_or(1);

    let items: Vec<ListItem> = app
        .stats
        .word_frequency
        .iter()
        .enumerate()
        .map(|(i, (word, count))| {
            let style = if i == app.selected_word {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let bar_width = 15;
            let filled = (*count * bar_width) / max_count;
            let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);

            let line = Line::from(vec![
                Span::styled(format!("{:>15} ", word), Style::default().fg(Color::Cyan)),
                Span::styled(bar, Style::default().fg(Color::Green)),
                Span::styled(format!(" {}", count), Style::default().fg(Color::Yellow)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Word Frequency (Top 20) "),
    );

    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
