pub mod theme;
pub mod screens;
pub mod components;

use crate::app::state::AppState;
use crate::player::Player;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

pub fn draw(frame: &mut Frame, app: &mut AppState, player: &Player) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.size());

    match app.screen.clone() {
        crate::app::state::Screen::Library => {
            screens::library::draw(frame, chunks[0], app, player);
        }
        crate::app::state::Screen::PlaylistDetail(_) => {
            screens::playlist_detail::draw(frame, chunks[0], app, player);
        }
        crate::app::state::Screen::AlbumDetail(ref album) => {
            screens::album_detail::draw(frame, chunks[0], album);
        }
        crate::app::state::Screen::ArtistDetail(ref artist) => {
            screens::artist_detail::draw(frame, chunks[0], artist);
        }
        crate::app::state::Screen::Search => {
            screens::search::draw(frame, chunks[0], app, player);
        }
        crate::app::state::Screen::NowPlaying => {
            screens::now_playing::draw(frame, chunks[0], app, player);
        }
        crate::app::state::Screen::Queue => {
            screens::queue::draw(frame, chunks[0], app);
        }
    }

    components::player_bar::draw(frame, chunks[1], app, player);

    if app.show_help {
        components::help::draw(frame);
    }
}
