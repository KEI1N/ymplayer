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

    let (liked, playlists) = tokio::join!(client.get_liked_tracks(0, 50), client.get_playlists());
    match liked {
        Ok(tracks) => app.liked_tracks = tracks,
        Err(e) => eprintln!("Ошибка загрузки: {e}"),
    }
    match playlists {
        Ok(playlists) => app.playlists = playlists,
        Err(e) => eprintln!("Ошибка загрузки плейлистов: {e}"),
    }

    // Search responses arrive here from spawned tasks, tagged with the
    // request generation so stale responses can be dropped.
    let (search_tx, mut search_rx) = tokio::sync::mpsc::unbounded_channel::<(u64, Vec<YTrack>)>();

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
                        if let Some(action) = Action::from_key(key, true) {
                            if handle_action(&mut app, &player, &client, &config, &search_tx, action).await? == false {
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

async fn handle_action(
    app: &mut AppState,
    player: &Player,
    client: &YandexClient,
    config: &Config,
    search_tx: &tokio::sync::mpsc::UnboundedSender<(u64, Vec<YTrack>)>,
    action: Action,
) -> Result<bool> {
    // ── Search mode: capture input ──
    if matches!(app.screen, Screen::Search) {
        match action {
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
            Action::Char(c) => {
                app.search_query.push(c);
            }
            Action::Search => {
                app.search_query.push('/');
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
                                        app.scroll_offset = 0;
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
        }

        Action::BackTab => {
            let views = LibraryView::all();
            let idx = views.iter().position(|v| *v == app.library_view).unwrap_or(0);
            let prev = if idx == 0 { views.len() - 1 } else { idx - 1 };
            app.library_view = views[prev].clone();
            app.selected_index = 0;
            app.sidebar_selected = prev;
            app.focus = Focus::Sidebar;
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

        Action::ScrollDown => {
            app.scroll_offset = (app.scroll_offset + 1).min(app.max_scroll());
        }

        Action::ScrollUp => {
            app.scroll_offset = app.scroll_offset.saturating_sub(1);
        }

        Action::PageDown => {
            app.scroll_offset = (app.scroll_offset + 10).min(app.max_scroll());
        }

        Action::PageUp => {
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
        }

        _ => {}
    }
    Ok(true)
}
