use crate::utils::{helpers::Quality, Result, YPlayerError};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub volume: u8,
    pub default_quality: String,
    pub crossfade_ms: u32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            volume: 80,
            default_quality: "flac".to_string(),
            crossfade_ms: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub show_cover_art: bool,
    pub cover_art_protocol: String,
    pub vim_keybindings: bool,
    pub language: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            show_cover_art: true,
            cover_art_protocol: "sixel".to_string(),
            vim_keybindings: true,
            language: "ru".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auto_login: bool,
    pub token_path: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            auto_login: true,
            token_path: "~/.config/yplayer/token.json".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpvConfig {
    pub extra_args: Vec<String>,
}

impl Default for MpvConfig {
    fn default() -> Self {
        Self {
            extra_args: vec![
                "--no-video".to_string(),
                "--msg-level=all=warn".to_string(),
                "--force-seekable=yes".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub player: PlayerConfig,
    pub ui: UiConfig,
    pub auth: AuthConfig,
    pub mpv: MpvConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            player: PlayerConfig::default(),
            ui: UiConfig::default(),
            auth: AuthConfig::default(),
            mpv: MpvConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir = config_dir()
            .ok_or_else(|| YPlayerError::Config("Config directory not found".into()))?;
        Ok(config_dir.join("yplayer").join("config.toml"))
    }

    pub fn token_path(&self) -> PathBuf {
        let expanded = if self.auth.token_path.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                self.auth.token_path.replacen("~", &home.to_string_lossy(), 1)
            } else {
                self.auth.token_path.clone()
            }
        } else {
            self.auth.token_path.clone()
        };
        PathBuf::from(expanded)
    }

    pub fn default_quality(&self) -> Quality {
        Quality::from_str(&self.player.default_quality)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
    pub device_code: Option<String>,
    pub user_code: Option<String>,
    pub verification_url: Option<String>,
}

impl TokenData {
    pub fn load(path: &Path) -> Result<Option<Self>> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let token: TokenData = serde_json::from_str(&content)?;
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = chrono::Utc::now().timestamp();
            now >= expires_at - 60
        } else {
            false
        }
    }
}