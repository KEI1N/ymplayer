use crate::app::state::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, _app: &AppState) {
    let text = vec![
        Line::from(" Очередь"),
        Line::from(" (TODO)"),
    ];
    let p = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Очередь"));
    frame.render_widget(p, area);
}