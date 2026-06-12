use crate::app::state::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, app: &AppState) {
    if let crate::app::state::Screen::PlaylistDetail(ref pl) = app.screen {
        let mut lines = vec![Line::from(format!(" {} — {} треков", pl.title, pl.track_count))
            .style(Style::default().add_modifier(Modifier::BOLD))];

        let start = app.scroll_offset.min(pl.tracks.len().saturating_sub(1));
        let visible = (area.height.saturating_sub(2) as usize).max(1);
        let end = (start + visible).min(pl.tracks.len());

        for i in start..end {
            let t = &pl.tracks[i];
            let prefix = if i == app.selected_index { "▸ " } else { "  " };
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Rgb(220, 220, 220))
                    .bg(Color::Rgb(60, 60, 120))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            lines.push(
                Line::from(format!(
                    "{}{:>3}. {} — {} [{}]",
                    prefix,
                    i + 1,
                    t.artists_str(),
                    t.title,
                    t.duration_str()
                ))
                .style(style),
            );
        }
        let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(pl.title.as_str()));
        frame.render_widget(p, area);
    }
}