use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, View};

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

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    render_folders(frame, app, main_chunks[0]);
    render_bookmarks(frame, app, main_chunks[1]);

    if app.view == View::Search {
        render_search(frame, app, chunks[1]);
    }

    render_status(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "BOOKMARK MANAGER",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} bookmarks", app.bookmarks.len()),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Bookmarks "));

    frame.render_widget(header, area);
}

fn render_folders(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .folders
        .iter()
        .enumerate()
        .map(|(i, folder)| {
            let style = if i == app.selected_folder && app.view == View::Folders {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else if i == app.selected_folder {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::raw("📁 "),
                Span::styled(&folder.name, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    format!("({})", folder.bookmarks.len()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let border_style = if app.view == View::Folders {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Folders ")
            .border_style(border_style),
    );

    frame.render_widget(list, area);
}

fn render_bookmarks(frame: &mut Frame, app: &App, area: Rect) {
    let visible = app.visible_bookmarks();

    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, bookmark)| {
            let style = if i == app.selected_bookmark && app.view == View::Bookmarks {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let tags_str = if bookmark.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", bookmark.tags.join(", "))
            };

            let line = Line::from(vec![
                Span::raw("🔗 "),
                Span::styled(&bookmark.title, Style::default().fg(Color::White)),
                Span::styled(tags_str, Style::default().fg(Color::Cyan)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let border_style = if app.view == View::Bookmarks {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let title = if !app.search_query.is_empty() {
        format!(" Results ({}) ", visible.len())
    } else {
        format!(" Bookmarks ({}) ", visible.len())
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style),
    );

    frame.render_widget(list, area);
}

fn render_search(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(50, 20, area);

    let search = Paragraph::new(format!("🔍 {}_", app.search_query))
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search Bookmarks ")
                .style(Style::default().bg(Color::Black)),
        );

    frame.render_widget(search, popup_area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {} ", app.status_text()))
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Length(3),
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
