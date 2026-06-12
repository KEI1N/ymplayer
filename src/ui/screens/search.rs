use crate::app::state::AppState;
use crate::api::models::YTrack;
use crate::ui::components::list::track_line;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut AppState) {
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

    let results = draw_results(&app.search_results, app.selected_index);
    let p = Paragraph::new(results)
        .block(Block::default().borders(Borders::ALL).title(format!("Результаты ({})", app.search_results.len())));
    frame.render_widget(p, chunks[1]);
}

fn draw_results<'a>(tracks: &'a [YTrack], selected: usize) -> Vec<Line<'a>> {
    if tracks.is_empty() {
        return vec![Line::from(" Начните вводить запрос")];
    }
    let mut lines = Vec::with_capacity(tracks.len() + 1);
    lines.push(
        Line::from(format!(" Найдено треков: {}", tracks.len()))
            .style(Style::default().add_modifier(Modifier::BOLD)),
    );
    for (i, t) in tracks.iter().enumerate() {
        lines.push(track_line(i, t, i == selected, true));
    }
    lines
}
