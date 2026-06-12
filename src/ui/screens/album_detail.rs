use crate::api::models::YAlbum;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, album: &YAlbum) {
    let mut text = vec![
        Line::from(format!(" {} — {} ({})", album.title, album.artists_str(), album.year.unwrap_or(0))),
        Line::from(format!(" {} треков | {}", album.track_count, album.genre.as_deref().unwrap_or("—"))),
        Line::from(""),
    ];
    for (i, t) in album.tracks.iter().enumerate() {
        text.push(Line::from(format!(
            " {:>3}. {} — {} [{}]",
            i + 1,
            t.title,
            t.artists_str(),
            t.duration_str()
        )));
    }
    let p = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(album.title.as_str()));
    frame.render_widget(p, area);
}