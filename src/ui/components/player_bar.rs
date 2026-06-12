use crate::app::state::AppState;
use crate::player::Player;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, _app: &AppState, player: &Player) {
    let track = player.current_track();
    let vol = player.volume();
    let playing = player.is_playing();
    let duration = player.duration().unwrap_or(0.0);
    let time_pos = player.time_pos().unwrap_or(0.0);
    let progress_pct = player.progress_pct();
    let repeat_mode = player.repeat_mode();
    let shuffle = player.shuffle_enabled();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let info = match track {
        Some(ref t) => format!(" {} — {} ", t.artists_str(), t.title),
        None => " — нет трека — ".to_string(),
    };

    let status = if playing { "▶" } else { "⏸" };
    let shuffle_icon = if shuffle { " 🔀" } else { "" };
    let repeat_icon = match repeat_mode {
        crate::player::queue::RepeatMode::One => " 🔂",
        crate::player::queue::RepeatMode::All => " 🔁",
        _ => "",
    };

    let line1 = format!(" {} {}{}{} ", status, info, shuffle_icon, repeat_icon);
    let p1 = Paragraph::new(line1).style(Style::default().fg(Color::White));
    frame.render_widget(p1, chunks[0]);

    let vol_filled = (vol / 5).min(20) as usize;
    let vol_bar = "█".repeat(vol_filled);
    let vol_empty = "░".repeat(20 - vol_filled);
    let vol_display = format!(" Vol: {:>3}% [{}{}] ", vol, vol_bar, vol_empty);

    let time_str = format!(
        "{:02}:{:02} / {:02}:{:02}",
        (time_pos as u32) / 60,
        (time_pos as u32) % 60,
        (duration as u32) / 60,
        (duration as u32) % 60
    );

    let progress_gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .ratio(progress_pct / 100.0)
        .label(time_str);

    let progress_area = chunks[1];
    frame.render_widget(progress_gauge, progress_area);

    let p3 = Paragraph::new(vol_display).style(Style::default().fg(Color::Gray));
    frame.render_widget(p3, chunks[2]);
}