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
    pub history: Vec<YTrack>,
    /// История загружается лениво при первом входе во вкладку.
    pub history_loaded: bool,
    pub history_loading: bool,
    pub current_playlist: Option<YPlaylist>,
    pub search_query: String,
    pub search_results: Vec<YTrack>,
    /// Generation counter for in-flight search requests; stale responses are dropped.
    pub search_seq: u64,
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
            history: vec![],
            history_loaded: false,
            history_loading: false,
            current_playlist: None,
            search_query: String::new(),
            search_results: vec![],
            search_seq: 0,
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
            LibraryView::History => &self.history,
        }
    }

    /// Number of rows in the Library content pane for the active view.
    /// In the Playlists view the pane lists playlists, not tracks.
    pub fn content_len(&self) -> usize {
        match self.library_view {
            LibraryView::Playlists => self.playlists.len(),
            _ => self.current_tracks().len(),
        }
    }

    /// Upper bound for scroll_offset on the current screen.
    pub fn max_scroll(&self) -> usize {
        match &self.screen {
            Screen::PlaylistDetail(pl) => pl.tracks.len().saturating_sub(1),
            _ => 0,
        }
    }
}
