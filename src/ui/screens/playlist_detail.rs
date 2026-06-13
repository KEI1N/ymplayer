use crate::app::state::AppState;
use crate::player::Player;
use crate::ui::components::list::{track_line, visible_window};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, app: &AppState, player: &Player) {
    if let crate::app::state::Screen::PlaylistDetail(ref pl) = app.screen {
        let mut lines = vec![Line::from(format!(" {} — {} треков", pl.title, pl.track_count))
            .style(Style::default().add_modifier(Modifier::BOLD))];

        let visible = (area.height.saturating_sub(3) as usize).max(1);
        let (start, end) = visible_window(pl.tracks.len(), app.selected_index, visible);

        for i in start..end {
            let t = &pl.tracks[i];
            lines.push(track_line(i, t, i == app.selected_index, true, player.is_cached(&t.id)));
        }
        let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(pl.title.as_str()));
        frame.render_widget(p, area);
    }
}