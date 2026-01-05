//! UI rendering for flashcard trainer.

use crate::app::{App, InputField, View};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    match app.view {
        View::DeckList => draw_deck_list(f, app),
        View::Study => draw_study(f, app),
        View::Stats => draw_stats(f, app),
        View::CardBrowser => draw_browser(f, app),
    }

    if app.show_help {
        draw_help(f);
    }

    if app.editing {
        draw_input(f, app);
    }

    if let Some(msg) = &app.message {
        draw_message(f, msg);
    }
}

fn draw_deck_list(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    // Header
    let header = Paragraph::new("Flashcard Trainer")
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Deck list
    if app.decks.is_empty() {
        let msg = Paragraph::new("No decks yet. Press 'a' to create one.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Decks "));
        f.render_widget(msg, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .decks
            .iter()
            .enumerate()
            .map(|(i, deck)| {
                let stats = app.deck_stats.get(&deck.id);
                let due = stats.map(|s| s.due_today).unwrap_or(0);
                let new = stats.map(|s| s.new_cards).unwrap_or(0);
                let total = stats.map(|s| s.total_cards).unwrap_or(0);

                let style = if i == app.selected_deck {
                    Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let spans = vec![
                    Span::styled(&deck.name, style),
                    Span::raw(" - "),
                    Span::styled(format!("{} due", due), Style::default().fg(Color::Yellow)),
                    Span::raw(", "),
                    Span::styled(format!("{} new", new), Style::default().fg(Color::Blue)),
                    Span::raw(format!(" ({} total)", total)),
                ];

                ListItem::new(Line::from(spans)).style(if i == app.selected_deck {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                })
            })
            .collect();

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Decks "));
        f.render_widget(list, chunks[1]);
    }

    // Footer
    let footer = Paragraph::new("j/k:Navigate  Enter:Study  a:Add deck  s:Stats  b:Browse  ?:Help  q:Quit")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

fn draw_study(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Progress
            Constraint::Min(0),     // Card
            Constraint::Length(5),  // Buttons
        ])
        .split(f.area());

    // Progress bar
    if let Some(session) = &app.session {
        let _progress = if session.total_cards() > 0 {
            session.current_index as f64 / session.total_cards() as f64
        } else {
            0.0
        };
        let progress_text = format!(
            "Card {} of {} | Accuracy: {:.0}%",
            session.current_index + 1,
            session.total_cards(),
            session.accuracy() * 100.0
        );
        let progress_bar = Paragraph::new(progress_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(progress_bar, chunks[0]);
    }

    // Card content
    let card_area = chunks[1];
    if let Some(card) = &app.current_card {
        if let Some(session) = &app.session {
            if session.flipped {
                // Show both front and back
                let inner = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(card_area);

                let front = Paragraph::new(card.front.as_str())
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).title(" Question "))
                    .wrap(Wrap { trim: true });
                f.render_widget(front, inner[0]);

                let back = Paragraph::new(card.back.as_str())
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Green))
                    .block(Block::default().borders(Borders::ALL).title(" Answer "))
                    .wrap(Wrap { trim: true });
                f.render_widget(back, inner[1]);
            } else {
                // Show only front
                let front = Paragraph::new(card.front.as_str())
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).title(" Question "))
                    .wrap(Wrap { trim: true });
                f.render_widget(front, card_area);
            }
        }
    } else if app.session.as_ref().map_or(false, |s| s.is_complete()) {
        let complete = Paragraph::new("Session complete! Press any key to continue.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(complete, card_area);
    }

    // Buttons
    let session = app.session.as_ref();
    let flipped = session.map_or(false, |s| s.flipped);

    let buttons = if flipped {
        vec![
            ("1", "Again", Color::Red),
            ("2", "Hard", Color::Yellow),
            ("3", "Good", Color::Green),
            ("4", "Easy", Color::Blue),
        ]
    } else {
        vec![("Space", "Show Answer", Color::White)]
    };

    let button_spans: Vec<Span> = buttons
        .iter()
        .flat_map(|(key, label, color)| {
            vec![
                Span::styled(format!("[{}]", key), Style::default().fg(*color).add_modifier(Modifier::BOLD)),
                Span::raw(format!(" {} ", label)),
                Span::raw("  "),
            ]
        })
        .collect();

    let button_line = Paragraph::new(Line::from(button_spans))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(button_line, chunks[2]);
}

fn draw_stats(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let header = Paragraph::new("Statistics")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    let rows: Vec<Row> = app
        .decks
        .iter()
        .map(|deck| {
            let stats = app.deck_stats.get(&deck.id);
            Row::new(vec![
                deck.name.clone(),
                stats.map(|s| s.total_cards.to_string()).unwrap_or_default(),
                stats.map(|s| s.due_today.to_string()).unwrap_or_default(),
                stats.map(|s| s.new_cards.to_string()).unwrap_or_default(),
                stats.map(|s| format!("{:.2}", s.average_ease)).unwrap_or_default(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
        ],
    )
    .header(Row::new(vec!["Deck", "Total", "Due", "New", "Avg Ease"]).style(Style::default().add_modifier(Modifier::BOLD)))
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(table, chunks[1]);

    let footer = Paragraph::new("q:Back  ?:Help")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

fn draw_browser(f: &mut Frame, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let header = Paragraph::new("Card Browser")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    let content = Paragraph::new("Card browser not yet implemented.\nPress 'a' to add a card.")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(content, chunks[1]);

    let footer = Paragraph::new("a:Add card  q:Back  ?:Help")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 80, f.area());
    f.render_widget(Clear, area);

    let help = r#"
Flashcard Trainer Keybindings

Deck List:
  j/k, Up/Down    Navigate decks
  Enter, Space    Start study session
  a               Add new deck
  s               View statistics
  b               Browse cards
  q               Quit

Study Session:
  Space           Show answer
  1               Again (failed)
  2               Hard
  3               Good
  4               Easy
  q, Esc          End session

General:
  ?               Show this help

Press any key to close
"#;

    let popup = Paragraph::new(help)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .wrap(Wrap { trim: false });
    f.render_widget(popup, area);
}

fn draw_input(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 15, f.area());
    f.render_widget(Clear, area);

    let title = match app.input_field {
        InputField::DeckName => "Enter deck name",
        InputField::CardFront => "Enter card front",
        InputField::CardBack => "Enter card back",
        InputField::None => "",
    };

    let input = Paragraph::new(app.input_buffer.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", title)));
    f.render_widget(input, area);

    f.set_cursor_position((area.x + 1 + app.input_buffer.len() as u16, area.y + 1));
}

fn draw_message(f: &mut Frame, msg: &str) {
    let area = Rect::new(
        f.area().x + 2,
        f.area().height.saturating_sub(5),
        f.area().width.saturating_sub(4),
        3,
    );
    f.render_widget(Clear, area);

    let message = Paragraph::new(msg)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(message, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
