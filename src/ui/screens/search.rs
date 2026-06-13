use crate::app::state::AppState;
use crate::api::models::YTrack;
use crate::player::Player;
use crate::ui::components::list::{track_line, visible_window};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut AppState, player: &Player) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let query_display = if app.search_query.is_empty() {
        " Введите запрос...".to_string()
    } else {
        format!(" {}", app.search_query)
    };

    let input = Paragraph::new(query_display)
        .block(Block::default().borders(Borders::ALL).title("Поиск"))
        .style(if app.search_query.is_empty() {
            Style::default().fg(Color::Rgb(120, 120, 120))
        } else {
            Style::default()
        });
    frame.render_widget(input, chunks[0]);

    let results = draw_results(&app.search_results, app.selected_index, chunks[1], player);
    let p = Paragraph::new(results)
        .block(Block::default().borders(Borders::ALL).title(format!("Результаты ({})", app.search_results.len())));
    frame.render_widget(p, chunks[1]);
}

fn draw_results(tracks: &[YTrack], selected: usize, area: Rect, player: &Player) -> Vec<Line<'static>> {
    if tracks.is_empty() {
        return vec![Line::from(" Начните вводить запрос")];
    }
    let mut lines = vec![
        Line::from(format!(" Найдено треков: {}", tracks.len()))
            .style(Style::default().add_modifier(Modifier::BOLD)),
    ];
    let visible = (area.height.saturating_sub(3) as usize).max(1);
    let (start, end) = visible_window(tracks.len(), selected, visible);
    for i in start..end {
        let t = &tracks[i];
        lines.push(track_line(i, t, i == selected, true, player.is_cached(&t.id)));
    }
    lines
}
