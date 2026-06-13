#![allow(dead_code)]

mod api;
mod app;
mod player;
mod ui;
mod utils;

use crate::api::auth::AuthFlow;
use crate::api::client::YandexClient;
use crate::api::models::{LibraryView, YTrack};
use crate::app::config::{Config, TokenData};
use crate::app::keybindings::Action;
use crate::app::state::{AppState, Focus, Screen};
use crate::player::Player;
use crate::utils::Result;
use crossterm::event::{Event, EventStream};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;

    let (client, token) = authenticate(&config).await?;

    enable_raw_mode()?;
    let mut stderr = stdout();
    crossterm::execute!(stderr, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let player = Player::new(&token)?;
    let mut app = AppState::new();

    // Search responses arrive here from spawned tasks, tagged with the
    // request generation so stale responses can be dropped.
    let (search_tx, mut search_rx) = tokio::sync::mpsc::unbounded_channel::<(u64, Vec<YTrack>)>();
    // Listening history is fetched once, in the background, on first entry
    // into the History view.
    let (history_tx, mut history_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<YTrack>>();
    // Liked tracks beyond the first page stream in chunk by chunk.
    let (liked_tx, mut liked_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<YTrack>>();

    const LIKED_PAGE: usize = 50;
    let (liked_ids, playlists) = tokio::join!(client.get_liked_track_ids(), client.get_playlists());
    match playlists {
        Ok(playlists) => app.playlists = playlists,
        Err(e) => eprintln!("Ошибка загрузки плейлистов: {e}"),
    }
    match liked_ids {
        Ok(ids) => {
            // First page synchronously so the library isn't empty on the
            // first frame; the rest streams in via liked_rx.
            let (first, rest) = ids.split_at(ids.len().min(LIKED_PAGE));
            match client.get_liked_tracks_chunk(first).await {
                Ok(tracks) => app.liked_tracks = tracks,
                Err(e) => eprintln!("Ошибка загрузки: {e}"),
            }
            if !rest.is_empty() {
                let rest = rest.to_vec();
                let client = client.clone();
                let tx = liked_tx.clone();
                tokio::spawn(async move {
                    for chunk in rest.chunks(LIKED_PAGE) {
                        match client.get_liked_tracks_chunk(chunk).await {
                            Ok(tracks) => {
                                if tx.send(tracks).is_err() {
                                    return;
                                }
                            }
                            Err(_) => return,
                        }
                    }
                });
            }
        }
        Err(e) => eprintln!("Ошибка загрузки: {e}"),
    }

    let mut events = EventStream::new();
    let tick_rate = tokio::time::Duration::from_millis(500);
    let mut ticker = tokio::time::interval(tick_rate);

    let mut needs_redraw = true;

    loop {
        // Poll mpv events before drawing (eof detection, etc.)
        player.poll_events();

        if needs_redraw {
            terminal.draw(|frame| ui::draw(frame, &mut app, &player))?;
            needs_redraw = false;
        }

        // Handle pending next track (EOF auto-advance)
        if player.pending_next() {
            player.clear_pending_next();
            if let Some(next) = player.queue_next() {
                if let Err(e) = player.play_cached(next, &client, &config.default_quality()).await {
                    eprintln!("Ошибка воспроизведения: {e}");
                }
                needs_redraw = true;
            }
        }

        tokio::select! {
            event = events.next() => {
                match event {
                    Some(Ok(Event::Key(key))) => {
                        // In search mode printable keys are query input,
                        // not player commands.
                        let action = if app.screen == Screen::Search {
                            Action::from_key_text_input(key)
                        } else {
                            Action::from_key(key, true)
                        };
                        if let Some(action) = action {
                            if handle_action(&mut app, &player, &client, &config, &search_tx, &history_tx, action).await? == false {
                                break;
                            }
                            needs_redraw = true;
                        }
                    }
                    Some(Ok(Event::Resize(_, _))) => {
                        needs_redraw = true;
                    }
                    _ => {}
                }
            }
            Some((seq, tracks)) = search_rx.recv() => {
                if seq == app.search_seq && app.screen == Screen::Search {
                    app.search_results = tracks;
                    if app.selected_index >= app.search_results.len() {
                        app.selected_index = 0;
                    }
                    needs_redraw = true;
                }
            }
            Some(tracks) = history_rx.recv() => {
                app.history = tracks;
                app.history_loaded = true;
                needs_redraw = true;
            }
            Some(tracks) = liked_rx.recv() => {
                app.liked_tracks.extend(tracks);
                needs_redraw = true;
            }
            _ = ticker.tick() => {
                needs_redraw = true;
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(stdout(), LeaveAlternateScreen)?;
    println!("👋 До свидания!");
    Ok(())
}

async fn authenticate(config: &Config) -> Result<(YandexClient, String)> {
    let token_path = config.token_path();
    let token_data = TokenData::load(&token_path)?;

    let token = if let Some(ref token_data) = token_data {
        if !token_data.is_expired() {
            token_data.access_token.clone()
        } else {
            let auth = AuthFlow;
            let token = auth.get_token_interactive().await?;
            let token_data = TokenData {
                access_token: token.clone(),
                refresh_token: None,
                expires_at: None,
                device_code: None,
                user_code: None,
                verification_url: None,
            };
            token_data.save(&token_path)?;
            println!("✅ Токен сохранён!");
            token
        }
    } else {
        let auth = AuthFlow;
        let token = auth.get_token_interactive().await?;
        let token_data = TokenData {
            access_token: token.clone(),
            refresh_token: None,
            expires_at: None,
            device_code: None,
            user_code: None,
            verification_url: None,
        };
        token_data.save(&token_path)?;
        println!("✅ Токен сохранён!");
        token
    };

    let mut client = YandexClient::new(&token)?;
    client.init().await?;
    println!("✅ Авторизация успешна!");

    Ok((client, token))
}

/// Move the cursor by `delta`, clamped to the list shown on the current
/// screen; the views scroll to follow the cursor at draw time.
fn move_selection(app: &mut AppState, delta: isize) {
    let len = match &app.screen {
        Screen::PlaylistDetail(pl) => pl.tracks.len(),
        Screen::Search => app.search_results.len(),
        Screen::Library => match app.focus {
            Focus::Sidebar => LibraryView::all().len(),
            Focus::Content => app.content_len(),
        },
        _ => 0,
    };
    let sidebar = matches!(app.screen, Screen::Library) && app.focus == Focus::Sidebar;
    let target = if sidebar { &mut app.sidebar_selected } else { &mut app.selected_index };
    if len == 0 {
        *target = 0;
        return;
    }
    let cur = (*target).min(len - 1) as isize;
    *target = (cur + delta).clamp(0, len as isize - 1) as usize;
}

/// Kick off a one-time background fetch of listening history when the
/// History view becomes active; the result arrives via `history_tx`.
fn maybe_load_history(
    app: &mut AppState,
    client: &YandexClient,
    history_tx: &tokio::sync::mpsc::UnboundedSender<Vec<YTrack>>,
) {
    if app.library_view == LibraryView::History && !app.history_loaded && !app.history_loading {
        app.history_loading = true;
        let client = client.clone();
        let tx = history_tx.clone();
        tokio::spawn(async move {
            let tracks = client.get_history(100).await.unwrap_or_default();
            let _ = tx.send(tracks);
        });
    }
}

async fn handle_action(
    app: &mut AppState,
    player: &Player,
    client: &YandexClient,
    config: &Config,
    search_tx: &tokio::sync::mpsc::UnboundedSender<(u64, Vec<YTrack>)>,
    history_tx: &tokio::sync::mpsc::UnboundedSender<Vec<YTrack>>,
    action: Action,
) -> Result<bool> {
    // ── Help overlay swallows everything until closed ──
    if app.show_help {
        match action {
            Action::Help | Action::Escape | Action::GoBack | Action::Quit => app.show_help = false,
            _ => {}
        }
        return Ok(true);
    }

    // ── Search mode: capture input ──
    if matches!(app.screen, Screen::Search) {
        match action {
            Action::Quit => return Ok(false),
            Action::Escape => {
                app.screen = Screen::Library;
                app.search_query.clear();
                app.search_seq += 1;
                app.selected_index = 0;
                return Ok(true);
            }
            Action::GoBack => {
                if app.search_query.is_empty() {
                    app.screen = Screen::Library;
                    app.search_seq += 1;
                    app.selected_index = 0;
                    return Ok(true);
                } else {
                    app.search_query.pop();
                }
            }
            Action::Enter => {
                if !app.search_results.is_empty() && app.selected_index < app.search_results.len() {
                    let track = app.search_results[app.selected_index].clone();
                    player.load_queue(app.search_results.clone());
                    if let Err(e) = player.play_cached(track, client, &config.default_quality()).await {
                        eprintln!("Ошибка: {e}");
                    }
                }
                app.screen = Screen::Library;
                app.search_query.clear();
                app.search_seq += 1;
                app.selected_index = 0;
                return Ok(true);
            }
            Action::Up => {
                if !app.search_results.is_empty() {
                    app.selected_index = app.selected_index.saturating_sub(1);
                }
                return Ok(true);
            }
            Action::Down => {
                if app.selected_index + 1 < app.search_results.len() {
                    app.selected_index += 1;
                }
                return Ok(true);
            }
            Action::PageUp => {
                move_selection(app, -10);
                return Ok(true);
            }
            Action::PageDown => {
                move_selection(app, 10);
                return Ok(true);
            }
            Action::Char(c) => {
                app.search_query.push(c);
            }
            _ => return Ok(true),
        }
        // The query changed: fire the search in the background so typing
        // never blocks the event loop; the result arrives via search_tx
        // and is dropped if a newer request superseded it.
        app.search_seq += 1;
        if app.search_query.is_empty() {
            app.search_results.clear();
        } else {
            let seq = app.search_seq;
            let query = app.search_query.clone();
            let client = client.clone();
            let tx = search_tx.clone();
            tokio::spawn(async move {
                if let Ok(results) = client.search(&query, "track", 0).await {
                    let _ = tx.send((seq, results.tracks));
                }
            });
        }
        return Ok(true);
    }

    // ── Normal mode ──
    match action {
        Action::Quit => return Ok(false),

        Action::Escape => {
            app.screen = Screen::Library;
        }

        Action::Up => {
            match app.screen {
                Screen::Library => {
                    match app.focus {
                        Focus::Sidebar => {
                            if app.sidebar_selected > 0 {
                                app.sidebar_selected -= 1;
                            }
                        }
                        Focus::Content => {
                            if app.selected_index > 0 {
                                app.selected_index -= 1;
                            }
                        }
                    }
                }
                Screen::PlaylistDetail(ref pl) => {
                    if !pl.tracks.is_empty() && app.selected_index > 0 {
                        app.selected_index -= 1;
                    }
                }
                Screen::NowPlaying => {
                    app.now_playing_scroll = app.now_playing_scroll.saturating_sub(1);
                }
                _ => {}
            }
        }

        Action::Down => {
            match app.screen {
                Screen::Library => {
                    match app.focus {
                        Focus::Sidebar => {
                            let views = LibraryView::all();
                            if app.sidebar_selected + 1 < views.len() {
                                app.sidebar_selected += 1;
                            }
                        }
                        Focus::Content => {
                            let max = app.content_len();
                            if max > 0 && app.selected_index + 1 < max {
                                app.selected_index += 1;
                            }
                        }
                    }
                }
                Screen::PlaylistDetail(ref pl) => {
                    if !pl.tracks.is_empty() && app.selected_index + 1 < pl.tracks.len() {
                        app.selected_index += 1;
                    }
                }
                Screen::NowPlaying => {
                    app.now_playing_scroll += 1;
                }
                _ => {}
            }
        }

        Action::Left => {
            match app.screen {
                Screen::Library => {
                    if app.focus == Focus::Content {
                        app.focus = Focus::Sidebar;
                    }
                }
                Screen::NowPlaying => {
                    app.screen = Screen::Library;
                }
                _ => {
                    app.screen = Screen::Library;
                }
            }
        }

        Action::Right => {
            if app.screen == Screen::Library {
                app.focus = Focus::Content;
            }
        }

        Action::Enter => {
            match app.screen {
                Screen::Library => {
                    match app.focus {
                        Focus::Sidebar => {
                            let views = LibraryView::all();
                            if app.sidebar_selected < views.len() {
                                let view = views[app.sidebar_selected].clone();
                                app.library_view = view;
                                app.selected_index = 0;
                                app.focus = Focus::Content;
                                maybe_load_history(app, client, history_tx);
                            }
                        }
                        Focus::Content => {
                            match app.library_view {
                                LibraryView::Playlists => {
                                    if app.selected_index < app.playlists.len() {
                                        let mut pl = app.playlists[app.selected_index].clone();
                                        let kind = pl.kind;
                                        if let Ok(tracks) = client.get_playlist_tracks(kind).await {
                                            pl.tracks = tracks;
                                        }
                                        app.current_playlist = Some(pl.clone());
                                        app.screen = Screen::PlaylistDetail(pl);
                                        app.selected_index = 0;
                                    }
                                }
                                _ => {
                                    let tracks = app.current_tracks().to_vec();
                                    if !tracks.is_empty() && app.selected_index < tracks.len() {
                                        let track = tracks[app.selected_index].clone();
                                        player.load_queue(tracks);
                                        player.play_cached(track, client, &config.default_quality()).await?;
                                    }
                                }
                            }
                        }
                    }
                }
                Screen::PlaylistDetail(ref mut pl) => {
                    if !pl.tracks.is_empty() && app.selected_index < pl.tracks.len() {
                        let track = pl.tracks[app.selected_index].clone();
                        player.load_queue(pl.tracks.clone());
                        player.play_cached(track, client, &config.default_quality()).await?;
                    }
                }
                _ => {}
            }
        }

        Action::Tab => {
            let views = LibraryView::all();
            let idx = views.iter().position(|v| *v == app.library_view).unwrap_or(0);
            let next = (idx + 1) % views.len();
            app.library_view = views[next].clone();
            app.selected_index = 0;
            app.sidebar_selected = next;
            app.focus = Focus::Sidebar;
            maybe_load_history(app, client, history_tx);
        }

        Action::BackTab => {
            let views = LibraryView::all();
            let idx = views.iter().position(|v| *v == app.library_view).unwrap_or(0);
            let prev = if idx == 0 { views.len() - 1 } else { idx - 1 };
            app.library_view = views[prev].clone();
            app.selected_index = 0;
            app.sidebar_selected = prev;
            app.focus = Focus::Sidebar;
            maybe_load_history(app, client, history_tx);
        }

        Action::PlayPause => {
            if player.current_track().is_some() {
                player.toggle_playback()?;
            } else {
                let tracks = app.current_tracks().to_vec();
                if !tracks.is_empty() && app.selected_index < tracks.len() {
                    let track = tracks[app.selected_index].clone();
                    player.load_queue(tracks);
                    player.play_cached(track, client, &config.default_quality()).await?;
                }
            }
        }

        Action::Next => {
            if let Some(next) = player.queue_next() {
                player.play_cached(next, client, &config.default_quality()).await?;
            }
        }

        Action::Prev => {
            if let Some(prev) = player.queue_prev() {
                player.play_cached(prev, client, &config.default_quality()).await?;
            }
        }

        Action::VolumeUp => {
            player.adjust_volume(5)?;
        }

        Action::VolumeDown => {
            player.adjust_volume(-5)?;
        }

        Action::SeekForward => {
            player.seek(5.0)?;
        }

        Action::SeekBackward => {
            player.seek(-5.0)?;
        }

        Action::ToggleShuffle => {
            player.toggle_shuffle();
        }

        Action::ToggleRepeat => {
            player.cycle_repeat();
        }

        Action::Search => {
            app.screen = Screen::Search;
            app.focus = Focus::Content;
            app.selected_index = 0;
        }

        Action::NowPlaying => {
            app.screen = Screen::NowPlaying;
        }

        Action::QueueView => {
            app.screen = Screen::Queue;
        }

        Action::PlayAll => {
            let tracks = app.current_tracks().to_vec();
            if !tracks.is_empty() {
                player.load_queue(tracks.clone());
                let track = tracks[0].clone();
                player.play_cached(track, client, &config.default_quality()).await?;
            }
        }

        Action::GoBack => {
            match app.screen {
                Screen::PlaylistDetail(_) | Screen::AlbumDetail(_) | Screen::ArtistDetail(_) | Screen::NowPlaying | Screen::Queue => {
                    app.screen = Screen::Library;
                }
                _ => {}
            }
        }

        Action::Help => {
            app.show_help = true;
        }

        Action::ScrollDown | Action::PageDown => {
            move_selection(app, 10);
        }

        Action::ScrollUp | Action::PageUp => {
            move_selection(app, -10);
        }

        _ => {}
    }
    Ok(true)
}
