use chrono::{DateTime, Duration, Local, TimeZone};
use std::fmt;

pub fn format_duration(ms: u32) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

pub fn format_duration_long(ms: u32) -> String {
    let total_seconds = ms / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

pub fn format_progress(current_ms: u32, total_ms: u32, width: u16) -> String {
    if total_ms == 0 {
        return "─".repeat(width as usize);
    }
    let ratio = current_ms as f32 / total_ms as f32;
    let filled = (ratio * width as f32) as usize;
    let empty = width as usize - filled;
    format!("{}{}", "█".repeat(filled), "─".repeat(empty))
}

pub fn format_number(n: u64) -> String {
    match n {
        0..=999 => n.to_string(),
        1000..=999_999 => format!("{:.1}K", n as f64 / 1000.0),
        1_000_000..=999_999_999 => format!("{:.1}M", n as f64 / 1_000_000.0),
        _ => format!("{:.1}B", n as f64 / 1_000_000_000.0),
    }
}

pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(1)).collect();
        format!("{}…", truncated)
    }
}

pub fn artist_names(artists: &[crate::api::models::YArtist]) -> String {
    artists
        .iter()
        .map(|a| a.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn timestamp_to_datetime(ts: i64) -> DateTime<Local> {
    Local.timestamp_opt(ts, 0).single().unwrap_or_else(Local::now)
}

pub fn format_relative_time(dt: DateTime<Local>) -> String {
    let now = Local::now();
    let diff = now.signed_duration_since(dt);

    if diff < Duration::minutes(1) {
        "только что".to_string()
    } else if diff < Duration::hours(1) {
        format!("{} мин. назад", diff.num_minutes())
    } else if diff < Duration::days(1) {
        format!("{} ч. назад", diff.num_hours())
    } else if diff < Duration::days(7) {
        format!("{} дн. назад", diff.num_days())
    } else {
        dt.format("%d.%m.%Y").to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Flac,
    Mp3_320,
    Aac,
    Any,
}

impl Quality {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "flac" => Quality::Flac,
            "mp3_320" | "mp3" => Quality::Mp3_320,
            "aac" => Quality::Aac,
            _ => Quality::Any,
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            Quality::Flac => 3,
            Quality::Mp3_320 => 2,
            Quality::Aac => 1,
            Quality::Any => 0,
        }
    }
}

impl fmt::Display for Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Quality::Flac => write!(f, "FLAC"),
            Quality::Mp3_320 => write!(f, "MP3 320"),
            Quality::Aac => write!(f, "AAC"),
            Quality::Any => write!(f, "Auto"),
        }
    }
}