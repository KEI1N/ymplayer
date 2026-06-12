use thiserror::Error;

#[derive(Error, Debug)]
pub enum YPlayerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Yandex Music API error: {0}")]
    YandexMusicApi(String),

    #[error("MPV error: {0}")]
    Mpv(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Playback error: {0}")]
    Playback(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("User cancelled")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, YPlayerError>;