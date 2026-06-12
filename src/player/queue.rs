use crate::api::models::YTrack;

#[derive(Debug, Clone)]
pub struct PlaybackQueue {
    tracks: Vec<YTrack>,
    current_index: usize,
    shuffle: bool,
    repeat: RepeatMode,
    shuffle_order: Vec<usize>,
    shuffle_position: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

impl PlaybackQueue {
    pub fn tracks(&self) -> &[YTrack] {
        &self.tracks
    }

    pub fn repeat(&self) -> &RepeatMode {
        &self.repeat
    }

    pub fn shuffle(&self) -> &bool {
        &self.shuffle
    }

    pub fn new() -> Self {
        Self {
            tracks: vec![],
            current_index: 0,
            shuffle: false,
            repeat: RepeatMode::Off,
            shuffle_order: vec![],
            shuffle_position: 0,
        }
    }

    pub fn append(&mut self, track: YTrack) {
        self.tracks.push(track);
    }

    pub fn append_multiple(&mut self, tracks: Vec<YTrack>) {
        self.tracks.extend(tracks);
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_index = 0;
        self.shuffle_order.clear();
        self.shuffle_position = 0;
    }

    /// Find track by ID and set it as current
    pub fn set_current_by_id(&mut self, track_id: &str) -> bool {
        if let Some(idx) = self.tracks.iter().position(|t| t.id == track_id) {
            self.current_index = idx;
            if self.shuffle {
                if let Some(pos) = self.shuffle_order.iter().position(|&i| i == idx) {
                    self.shuffle_position = pos;
                }
            }
            true
        } else {
            false
        }
    }

    pub fn current(&self) -> Option<&YTrack> {
        if self.shuffle {
            self.shuffle_order
                .get(self.shuffle_position)
                .and_then(|&i| self.tracks.get(i))
        } else {
            self.tracks.get(self.current_index)
        }
    }

    fn ensure_shuffle_order(&mut self) {
        if self.shuffle_order.len() != self.tracks.len() {
            self.regenerate_shuffle_order();
        }
    }

    fn regenerate_shuffle_order(&mut self) {
        let n = self.tracks.len();
        self.shuffle_order = (0..n).collect();
        for i in (1..n).rev() {
            let j = fastrand::usize(..=i);
            self.shuffle_order.swap(i, j);
        }
        self.shuffle_position = self
            .shuffle_order
            .iter()
            .position(|&i| i == self.current_index)
            .unwrap_or(0);
    }

    pub fn next_track(&mut self) -> Option<&YTrack> {
        if self.tracks.is_empty() {
            return None;
        }

        if self.shuffle {
            self.ensure_shuffle_order();
            if self.shuffle_position + 1 >= self.shuffle_order.len() {
                match self.repeat {
                    RepeatMode::All => {
                        // Wrap around into a fresh order, starting from its
                        // first track rather than the one that just finished
                        // (regenerate leaves shuffle_position on the current track).
                        self.regenerate_shuffle_order();
                        self.shuffle_position = 0;
                        if self.shuffle_order.first() == Some(&self.current_index)
                            && self.shuffle_order.len() > 1
                        {
                            let last = self.shuffle_order.len() - 1;
                            self.shuffle_order.swap(0, last);
                        }
                    }
                    RepeatMode::One => {}
                    RepeatMode::Off => return None,
                }
            } else {
                self.shuffle_position += 1;
            }
        } else {
            if self.current_index + 1 >= self.tracks.len() {
                match self.repeat {
                    RepeatMode::All => self.current_index = 0,
                    RepeatMode::One => {}
                    RepeatMode::Off => return None,
                }
            } else {
                self.current_index += 1;
            }
        }
        self.current()
    }

    pub fn prev_track(&mut self) -> Option<&YTrack> {
        if self.tracks.is_empty() {
            return None;
        }

        if self.shuffle {
            self.ensure_shuffle_order();
            if self.shuffle_position == 0 {
                match self.repeat {
                    RepeatMode::All => self.shuffle_position = self.shuffle_order.len().saturating_sub(1),
                    RepeatMode::One => {}
                    RepeatMode::Off => return None,
                }
            } else {
                self.shuffle_position -= 1;
            }
        } else {
            if self.current_index == 0 {
                match self.repeat {
                    RepeatMode::All => self.current_index = self.tracks.len().saturating_sub(1),
                    RepeatMode::One => {}
                    RepeatMode::Off => return None,
                }
            } else {
                self.current_index -= 1;
            }
        }
        self.current()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn set_shuffle(&mut self, shuffle: bool) {
        if shuffle == self.shuffle {
            return;
        }
        if shuffle {
            self.regenerate_shuffle_order();
        } else {
            if let Some(&idx) = self.shuffle_order.get(self.shuffle_position) {
                self.current_index = idx;
            }
            self.shuffle_order.clear();
            self.shuffle_position = 0;
        }
        self.shuffle = shuffle;
    }

    pub fn set_repeat(&mut self, mode: RepeatMode) {
        self.repeat = mode;
    }

    pub fn toggle_shuffle(&mut self) -> bool {
        self.set_shuffle(!self.shuffle);
        self.shuffle
    }

    pub fn cycle_repeat(&mut self) -> RepeatMode {
        self.repeat = match self.repeat {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        };
        self.repeat
    }
}