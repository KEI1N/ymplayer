use crate::api::models::{YTrack, YPlaylist, YAlbum, YArtist, LibraryView};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Library,
    PlaylistDetail(YPlaylist),
    AlbumDetail(YAlbum),
    ArtistDetail(YArtist),
    Search,
    NowPlaying,
    Queue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    Content,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub screen: Screen,
    pub focus: Focus,
    pub library_view: LibraryView,
    pub liked_tracks: Vec<YTrack>,
    pub playlists: Vec<YPlaylist>,
    pub current_playlist: Option<YPlaylist>,
    pub current_album: Option<YAlbum>,
    pub current_artist: Option<YArtist>,
    pub search_query: String,
    pub search_results: Vec<YTrack>,
    pub selected_index: usize,
    pub sidebar_selected: usize,
    pub scroll_offset: usize,
    pub now_playing_scroll: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Library,
            focus: Focus::Sidebar,
            library_view: LibraryView::LikedTracks,
            liked_tracks: vec![],
            playlists: vec![],
            current_playlist: None,
            current_album: None,
            current_artist: None,
            search_query: String::new(),
            search_results: vec![],
            selected_index: 0,
            sidebar_selected: 0,
            scroll_offset: 0,
            now_playing_scroll: 0,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_tracks(&self) -> &[YTrack] {
        match self.library_view {
            LibraryView::LikedTracks => &self.liked_tracks,
            LibraryView::Playlists => self.current_playlist.as_ref().map(|p| p.tracks.as_slice()).unwrap_or(&[]),
            _ => &[],
        }
    }
}
