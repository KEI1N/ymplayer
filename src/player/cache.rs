use crate::utils::{Result, YPlayerError};
use std::path::PathBuf;

const CACHE_DIR: &str = "tracks";

pub struct TrackCache {
    dir: PathBuf,
}

impl TrackCache {
    pub fn new() -> Result<Self> {
        let base = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("~/.cache"))
            .join("yplayer");
        let dir = base.join(CACHE_DIR);
        std::fs::create_dir_all(&dir)
            .map_err(|e| YPlayerError::Io(e))?;
        Ok(Self { dir })
    }

    pub fn path(&self, track_id: &str) -> PathBuf {
        // Track ids come from API responses; keep only filename-safe
        // characters so an id can't escape the cache dir via ".." or "/".
        let safe: String = track_id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
            .collect();
        self.dir.join(format!("{safe}.mp3"))
    }

    pub fn has(&self, track_id: &str) -> bool {
        self.path(track_id).exists()
    }

    pub fn get(&self, track_id: &str) -> Option<PathBuf> {
        let p = self.path(track_id);
        if p.exists() { Some(p) } else { None }
    }

    pub fn put(&self, track_id: &str, data: &[u8]) -> Result<PathBuf> {
        let path = self.path(track_id);
        std::fs::write(&path, data)
            .map_err(|e| YPlayerError::Io(e))?;
        Ok(path)
    }

    pub fn clear(&self) -> Result<()> {
        if self.dir.exists() {
            std::fs::remove_dir_all(&self.dir)
                .map_err(|e| YPlayerError::Io(e))?;
            std::fs::create_dir_all(&self.dir)
                .map_err(|e| YPlayerError::Io(e))?;
        }
        Ok(())
    }
}
