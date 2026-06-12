use crate::api::models::{LibraryView, YTrack, YPlaylist};
use crate::app::state::{AppState, Focus};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(1)])
        .split(area);

    draw_sidebar(frame, chunks[0], app);
    draw_content(frame, chunks[1], app);
}

fn draw_sidebar(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let focused = app.focus == Focus::Sidebar;
    let views = LibraryView::all();
    let items: Vec<ListItem> = views.iter().enumerate().map(|(i, v)| {
        let icon = v.icon();
        let name = v.name();
        let active = app.library_view == *v;
        let prefix = if active { "▸ " } else { "  " };
        let style = if focused && app.sidebar_selected == i {
            Style::default()
                .fg(Color::Rgb(220, 220, 220))
                .bg(Color::Rgb(60, 60, 120))
                .add_modifier(Modifier::BOLD)
        } else if active {
            Style::default().fg(Color::Rgb(100, 180, 255)).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(format!("{prefix}{icon} {name}")).style(style)
    }).collect();

    let title = if focused { "► Библиотека" } else { "  Библиотека" };
    let sidebar = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).title(title));

    frame.render_widget(sidebar, area);
}

fn draw_content(frame: &mut Frame, area: Rect, app: &AppState) {
    match app.library_view {
        LibraryView::LikedTracks => {
            draw_track_list(frame, area, &app.liked_tracks, "Мне нравится", app.selected_index, app.focus == Focus::Content);
        }
        LibraryView::Playlists => {
            draw_playlist_list(frame, area, &app.playlists, "Плейлисты", app.selected_index, app.focus == Focus::Content);
        }
        _ => {
            let p = Paragraph::new("Скоро")
                .block(Block::default().borders(Borders::ALL).title("Содержимое"));
            frame.render_widget(p, area);
        }
    }
}

fn draw_track_list(frame: &mut Frame, area: Rect, tracks: &[YTrack], title: &str, selected: usize, focused: bool) {
    let mut lines = vec![Line::from(format!(" {} ({} треков)", title, tracks.len()))
        .style(Style::default().add_modifier(Modifier::BOLD))];
    for (i, t) in tracks.iter().enumerate() {
        let prefix = if i == selected { "▸ " } else { "  " };
        let style = if focused && i == selected {
            Style::default()
                .fg(Color::Rgb(220, 220, 220))
                .bg(Color::Rgb(60, 60, 120))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(
            Line::from(format!(
                "{}{:>3}. {} — {} [{}]",
                prefix,
                i + 1,
                t.artists_str(),
                t.title,
                t.duration_str()
            ))
            .style(style),
        );
    }
    let title_line = if focused { "► Содержимое" } else { "  Содержимое" };
    let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title_line));
    frame.render_widget(p, area);
}

fn draw_playlist_list(frame: &mut Frame, area: Rect, playlists: &[YPlaylist], title: &str, selected: usize, focused: bool) {
    let mut lines = vec![Line::from(format!(" {} ({} плейлистов)", title, playlists.len()))
        .style(Style::default().add_modifier(Modifier::BOLD))];
    for (i, pl) in playlists.iter().enumerate() {
        let prefix = if i == selected { "▸ " } else { "  " };
        let style = if focused && i == selected {
            Style::default()
                .fg(Color::Rgb(220, 220, 220))
                .bg(Color::Rgb(60, 60, 120))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(
            Line::from(format!("{}{:>3}. {} — {} тр.", prefix, i + 1, pl.title, pl.track_count))
                .style(style),
        );
    }
    let title_line = if focused { "► Содержимое" } else { "  Содержимое" };
    let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title_line));
    frame.render_widget(p, area);
}
