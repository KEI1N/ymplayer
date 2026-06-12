use crate::api::models::YArtist;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, artist: &YArtist) {
    let text = vec![
        Line::from(format!(" {}", artist.name)),
        Line::from(format!(" Жанры: {}", artist.genres_str())),
        Line::from(format!(" Треков: {} | Альбомов: {}",
            artist.tracks_count.map(|c| c.to_string()).unwrap_or_else(|| "—".into()),
            artist.albums_count.map(|c| c.to_string()).unwrap_or_else(|| "—".into()),
        )),
        Line::from(""),
        Line::from(" (загрузка треков…)"),
    ];
    let p = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(artist.name.as_str()));
    frame.render_widget(p, area);
}