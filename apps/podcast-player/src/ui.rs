use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

use crate::app::{App, View};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);

    match app.view {
        View::Podcasts => render_podcasts(frame, app, chunks[1]),
        View::Episodes => render_episodes(frame, app, chunks[1]),
        View::Player => render_player_main(frame, app, chunks[1]),
    }

    render_now_playing(frame, app, chunks[2]);
    render_status(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let playing_indicator = if app.is_playing { "▶" } else { "⏸" };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "PODCAST PLAYER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(playing_indicator, Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::styled(
            format!("{:?}", app.view),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Podcasts "));

    frame.render_widget(header, area);
}

fn render_podcasts(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .podcasts
        .iter()
        .enumerate()
        .map(|(i, podcast)| {
            let style = if i == app.selected_podcast {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let unplayed = podcast.episodes.iter().filter(|e| !e.played).count();

            let line = Line::from(vec![
                Span::raw("🎙️ "),
                Span::styled(&podcast.title, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    format!("({} episodes, {} new)", podcast.episodes.len(), unplayed),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Subscriptions ({}) ", app.podcasts.len())),
    );

    frame.render_widget(list, area);
}

fn render_episodes(frame: &mut Frame, app: &App, area: Rect) {
    let Some(podcast) = app.podcasts.get(app.selected_podcast) else {
        return;
    };

    let items: Vec<ListItem> = podcast
        .episodes
        .iter()
        .enumerate()
        .map(|(i, episode)| {
            let style = if i == app.selected_episode {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let status_icon = if episode.played {
                "✓"
            } else if episode.progress > 0 {
                "◐"
            } else {
                "○"
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", status_icon), Style::default().fg(Color::Green)),
                Span::styled(&episode.title, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", episode.duration_formatted()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", podcast.title)),
    );

    frame.render_widget(list, area);
}

fn render_player_main(frame: &mut Frame, app: &App, area: Rect) {
    let Some(episode) = app.current_episode_ref() else {
        let placeholder = Paragraph::new("No episode selected")
            .block(Block::default().borders(Borders::ALL).title(" Player "))
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, area);
        return;
    };

    let podcast_title = app
        .current_podcast_ref()
        .map(|p| p.title.as_str())
        .unwrap_or("Unknown");

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Podcast: ", Style::default().fg(Color::Cyan)),
            Span::raw(podcast_title),
        ]),
        Line::from(vec![
            Span::styled("Episode: ", Style::default().fg(Color::Cyan)),
            Span::raw(&episode.title),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Progress: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!(
                "{} / {} ({:.1}%)",
                episode.progress_formatted(),
                episode.duration_formatted(),
                episode.progress_percent()
            )),
        ]),
        Line::from(vec![
            Span::styled("Volume: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}%", app.volume)),
        ]),
        Line::from(vec![
            Span::styled("Speed: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{:.2}x", app.playback_speed)),
        ]),
    ];

    let player = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Now Playing "));

    frame.render_widget(player, area);
}

fn render_now_playing(frame: &mut Frame, app: &App, area: Rect) {
    let Some(episode) = app.current_episode_ref() else {
        let empty = Paragraph::new("No episode playing")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(2)])
        .margin(1)
        .split(area);

    let block = Block::default().borders(Borders::ALL);
    frame.render_widget(block, area);

    let playing_indicator = if app.is_playing { "▶" } else { "⏸" };
    let title = Paragraph::new(Line::from(vec![
        Span::styled(playing_indicator, Style::default().fg(Color::Green)),
        Span::raw(" "),
        Span::raw(&episode.title),
    ]));
    frame.render_widget(title, chunks[0]);

    let progress = Gauge::default()
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent((episode.progress_percent()) as u16)
        .label(format!(
            "{} / {}",
            episode.progress_formatted(),
            episode.duration_formatted()
        ));
    frame.render_widget(progress, chunks[1]);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}
