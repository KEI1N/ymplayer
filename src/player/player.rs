use crate::api::client::YandexClient;
use crate::api::models::YTrack;
use crate::player::cache::TrackCache;
use crate::player::mpv::MpvPlayer;
use crate::player::queue::{PlaybackQueue, RepeatMode};
use crate::utils::Quality;
use crate::utils::Result;
use std::sync::Mutex;

pub struct Player {
    mpv: MpvPlayer,
    queue: Mutex<PlaybackQueue>,
    cache: TrackCache,
    current_track: Mutex<Option<YTrack>>,
    pending_next: Mutex<bool>,
    suppress_next_end_file: Mutex<bool>,
}

impl Player {
    pub fn new(token: &str) -> Result<Self> {
        let mpv = MpvPlayer::new(Some(token))?;
        let cache = TrackCache::new()?;
        Ok(Self {
            mpv,
            cache,
            queue: Mutex::new(PlaybackQueue::new()),
            current_track: Mutex::new(None),
            pending_next: Mutex::new(false),
            suppress_next_end_file: Mutex::new(false),
        })
    }

    pub fn current_track(&self) -> Option<YTrack> {
        self.current_track.lock().ok().and_then(|g| g.clone())
    }

    pub fn volume(&self) -> u8 {
        self.mpv.volume()
    }

    pub fn is_playing(&self) -> bool {
        !self.mpv.paused()
    }

    pub fn duration(&self) -> Option<f64> {
        self.mpv.duration()
    }

    pub fn time_pos(&self) -> Option<f64> {
        self.mpv.time_pos()
    }

    pub fn progress_pct(&self) -> f64 {
        let dur = self.mpv.duration().unwrap_or(1.0);
        let pos = self.mpv.time_pos().unwrap_or(0.0);
        if dur <= 0.0 { 0.0 } else { (pos / dur * 100.0).clamp(0.0, 100.0) }
    }

    pub fn play_track(&self, track: YTrack, url: &str) -> Result<()> {
        self.set_suppress_next_end_file(true);
        self.mpv.load(url)?;
        self.mpv.play()?;
        self.mpv.drain_events();
        self.clear_pending_next();
        self.sync_queue_current(&track.id);
        *self.current_track.lock().map_err(|e| crate::utils::YPlayerError::InvalidState(e.to_string()))? = Some(track);
        Ok(())
    }

    fn set_suppress_next_end_file(&self, val: bool) {
        if let Ok(mut v) = self.suppress_next_end_file.lock() {
            *v = val;
        }
    }

    fn take_suppress_next_end_file(&self) -> bool {
        self.suppress_next_end_file.lock().map(|mut v| std::mem::take(&mut *v)).unwrap_or(false)
    }

    fn sync_queue_current(&self, track_id: &str) {
        if let Ok(mut q) = self.queue.lock() {
            q.set_current_by_id(track_id);
        }
    }

    /// Play a track with caching: download to local cache, then play from disk.
    pub async fn play_cached(
        &self,
        track: YTrack,
        client: &YandexClient,
        quality: &Quality,
    ) -> Result<()> {
        let path = if self.cache.has(&track.id) {
            self.cache.get(&track.id).unwrap()
        } else {
            let url = client.get_best_download_url(&track.id, quality).await?;
            let bytes = client.download_audio(&url).await?;
            self.cache.put(&track.id, &bytes)?
        };

        let path_str = path.to_string_lossy().to_string();
        self.play_track(track, &path_str)
    }

    pub fn clear_cache(&self) -> Result<()> {
        self.cache.clear()
    }

    /// Whether the track is already downloaded to the local cache.
    pub fn is_cached(&self, track_id: &str) -> bool {
        self.cache.has(track_id)
    }

    pub fn toggle_playback(&self) -> Result<bool> {
        self.mpv.toggle_playback()
    }

    pub fn set_volume(&self, vol: u8) -> Result<()> {
        self.mpv.set_volume(vol)
    }

    pub fn adjust_volume(&self, delta: i8) -> Result<u8> {
        self.mpv.adjust_volume(delta)
    }

    pub fn seek(&self, secs: f64) -> Result<()> {
        self.mpv.seek(secs)
    }

    pub fn stop(&self) -> Result<()> {
        self.mpv.stop()?;
        *self.current_track.lock().map_err(|e| crate::utils::YPlayerError::InvalidState(e.to_string()))? = None;
        Ok(())
    }

    pub fn load_queue(&self, tracks: Vec<YTrack>) {
        if let Ok(mut q) = self.queue.lock() {
            q.clear();
            q.append_multiple(tracks);
        }
    }

    pub fn queue_append(&self, track: YTrack) {
        if let Ok(mut q) = self.queue.lock() {
            q.append(track);
        }
    }

    pub fn queue_len(&self) -> usize {
        self.queue.lock().map(|q| q.len()).unwrap_or(0)
    }

    pub fn queue_tracks(&self) -> Vec<YTrack> {
        self.queue.lock().map(|q| q.tracks().to_vec()).unwrap_or_default()
    }

    pub fn queue_next(&self) -> Option<YTrack> {
        self.queue.lock().ok().and_then(|mut q| q.next_track().cloned())
    }

    pub fn queue_prev(&self) -> Option<YTrack> {
        self.queue.lock().ok().and_then(|mut q| q.prev_track().cloned())
    }

    pub fn toggle_shuffle(&self) -> bool {
        self.queue.lock().map(|mut q| q.toggle_shuffle()).unwrap_or(false)
    }

    pub fn cycle_repeat(&self) -> RepeatMode {
        self.queue.lock().map(|mut q| q.cycle_repeat()).unwrap_or(RepeatMode::Off)
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.queue.lock().map(|q| *q.repeat()).unwrap_or(RepeatMode::Off)
    }

    pub fn shuffle_enabled(&self) -> bool {
        self.queue.lock().map(|q| *q.shuffle()).unwrap_or(false)
    }

    pub fn pending_next(&self) -> bool {
        self.pending_next.lock().map(|v| *v).unwrap_or(false)
    }

    pub fn clear_pending_next(&self) {
        if let Ok(mut v) = self.pending_next.lock() {
            *v = false;
        }
    }

    pub fn poll_events(&self) {
        use libmpv_sys::mpv_event_id_MPV_EVENT_END_FILE;
        while let Some(event) = self.mpv.poll_event() {
            if event == mpv_event_id_MPV_EVENT_END_FILE as i64 {
                if self.take_suppress_next_end_file() {
                    continue;
                }
                if let Ok(mut v) = self.pending_next.lock() {
                    *v = true;
                }
            }
        }
    }
}
