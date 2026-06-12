use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YTrack {
    pub id: String,
    pub title: String,
    pub artists: Vec<YArtist>,
    pub albums: Vec<YAlbum>,
    pub duration_ms: u32,
    pub cover_uri: Option<String>,
    pub lyrics_available: bool,
    pub explicit: bool,
    pub track_position: Option<u32>,
    pub liked: bool,
}

impl YTrack {
    pub fn artists_str(&self) -> String {
        self.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ")
    }

    pub fn album_str(&self) -> String {
        self.albums.first().map(|a| a.title.as_str()).unwrap_or("").to_string()
    }

    pub fn duration_str(&self) -> String {
        let total_secs = self.duration_ms / 1000;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    pub fn display_title(&self) -> String {
        format!("{} — {}", self.artists_str(), self.title)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YArtist {
    pub id: String,
    pub name: String,
    pub cover_uri: Option<String>,
    pub genres: Vec<String>,
    pub tracks_count: Option<u32>,
    pub albums_count: Option<u32>,
    pub liked: bool,
}

impl YArtist {
    pub fn genres_str(&self) -> String {
        self.genres.join(", ")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YAlbum {
    pub id: String,
    pub title: String,
    pub artists: Vec<YArtist>,
    pub cover_uri: Option<String>,
    pub track_count: u32,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub liked: bool,
    pub tracks: Vec<YTrack>,
}

impl YAlbum {
    pub fn artists_str(&self) -> String {
        self.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YPlaylist {
    pub id: String,
    pub kind: i64,
    pub title: String,
    pub owner_name: String,
    pub track_count: u32,
    pub cover_uri: Option<String>,
    pub description: Option<String>,
    pub duration_ms: u32,
    pub liked: bool,
    pub tracks: Vec<YTrack>,
}

impl YPlaylist {
    pub fn duration_str(&self) -> String {
        let total_secs = self.duration_ms / 1000;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YSearchResult {
    pub tracks: Vec<YTrack>,
    pub artists: Vec<YArtist>,
    pub albums: Vec<YAlbum>,
    pub playlists: Vec<YPlaylist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YRadioStation {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub icon_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YTrackDownloadInfo {
    pub url: String,
    pub codec: String,
    pub bitrate: u32,
    pub gain: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YLyrics {
    pub lyrics: String,
    pub has_timing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryView {
    LikedTracks,
    Playlists,
    History,
}

impl LibraryView {
    pub fn all() -> &'static [LibraryView] {
        &[
            LibraryView::LikedTracks,
            LibraryView::Playlists,
            LibraryView::History,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            LibraryView::LikedTracks => "Мне нравится",
            LibraryView::Playlists => "Плейлисты",
            LibraryView::History => "История",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            LibraryView::LikedTracks => "♥",
            LibraryView::Playlists => "♪",
            LibraryView::History => "⏱",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub interval: u32,
    pub expires_in: u32,
}

#[derive(Debug, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u32,
    pub token_type: String,
}