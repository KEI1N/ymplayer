use crate::api::models::{LibraryView, YTrack, YPlaylist};
use crate::app::state::{AppState, Focus};
use crate::player::Player;
use crate::ui::components::list::{row_prefix, row_style, track_line, visible_window};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut AppState, player: &Player) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(1)])
        .split(area);

    draw_sidebar(frame, chunks[0], app);
    draw_content(frame, chunks[1], app, player);
}

fn draw_sidebar(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let focused = app.focus == Focus::Sidebar;
    let views = LibraryView::all();
    let items: Vec<ListItem> = views.iter().enumerate().map(|(i, v)| {
        let icon = v.icon();
        let name = v.name();
        let active = app.library_view == *v;
        let prefix = row_prefix(active);
        let style = if focused && app.sidebar_selected == i {
            row_style(true)
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

fn draw_content(frame: &mut Frame, area: Rect, app: &AppState, player: &Player) {
    match app.library_view {
        LibraryView::LikedTracks => {
            draw_track_list(frame, area, &app.liked_tracks, "Мне нравится", app.selected_index, app.focus == Focus::Content, player);
        }
        LibraryView::Playlists => {
            draw_playlist_list(frame, area, &app.playlists, "Плейлисты", app.selected_index, app.focus == Focus::Content);
        }
        LibraryView::History => {
            if app.history_loaded {
                draw_track_list(frame, area, &app.history, "История", app.selected_index, app.focus == Focus::Content, player);
            } else {
                let p = Paragraph::new(" Загрузка истории...")
                    .block(Block::default().borders(Borders::ALL).title("История"));
                frame.render_widget(p, area);
            }
        }
    }
}

fn draw_track_list(frame: &mut Frame, area: Rect, tracks: &[YTrack], title: &str, selected: usize, focused: bool, player: &Player) {
    let mut lines = vec![Line::from(format!(" {} ({} треков)", title, tracks.len()))
        .style(Style::default().add_modifier(Modifier::BOLD))];
    // 2 border rows + 1 header row
    let visible = (area.height.saturating_sub(3) as usize).max(1);
    let (start, end) = visible_window(tracks.len(), selected, visible);
    for i in start..end {
        let t = &tracks[i];
        lines.push(track_line(i, t, i == selected, focused, player.is_cached(&t.id)));
    }
    let title_line = if focused { "► Содержимое" } else { "  Содержимое" };
    let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title_line));
    frame.render_widget(p, area);
}

fn draw_playlist_list(frame: &mut Frame, area: Rect, playlists: &[YPlaylist], title: &str, selected: usize, focused: bool) {
    let mut lines = vec![Line::from(format!(" {} ({} плейлистов)", title, playlists.len()))
        .style(Style::default().add_modifier(Modifier::BOLD))];
    let visible = (area.height.saturating_sub(3) as usize).max(1);
    let (start, end) = visible_window(playlists.len(), selected, visible);
    for i in start..end {
        let pl = &playlists[i];
        lines.push(
            Line::from(format!("{}{:>3}. {} — {} тр.", row_prefix(i == selected), i + 1, pl.title, pl.track_count))
                .style(row_style(focused && i == selected)),
        );
    }
    let title_line = if focused { "► Содержимое" } else { "  Содержимое" };
    let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title_line));
    frame.render_widget(p, area);
}
