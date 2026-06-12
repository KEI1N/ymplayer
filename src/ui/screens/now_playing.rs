use crate::app::state::AppState;
use crate::player::Player;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, _app: &AppState, player: &Player) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    let track = player.current_track();
    let text = match track {
        Some(ref t) => vec![
            Line::from(format!(" {}", t.title)).style(Style::default().add_modifier(Modifier::BOLD)),
            Line::from(format!(" {}", t.artists_str())),
            Line::from(format!(" {}", t.albums.first().map(|a| a.title.as_str()).unwrap_or(""))),
        ],
        None => vec![Line::from(" Нет активного трека")],
    };

    let block = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Сейчас играет"));
    frame.render_widget(block, chunks[0]);

    let playing = player.is_playing();
    let icon = if playing { "▶" } else { "⏸" };
    let pos = player.time_pos().unwrap_or(0.0);
    let dur = player.duration().unwrap_or(0.0);
    let pos_fmt = format!("{:02}:{:02}", (pos as u32) / 60, (pos as u32) % 60);
    let dur_fmt = format!("{:02}:{:02}", (dur as u32) / 60, (dur as u32) % 60);
    let label_str = format!(" {}  {} / {}  Vol: {}% {} ", icon, pos_fmt, dur_fmt, player.volume(), vol_bar(player.volume()));

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Rgb(60, 120, 200)))
        .percent(player.progress_pct() as u16)
        .label(label_str);
    frame.render_widget(gauge, chunks[1]);
}

fn vol_bar(vol: u8) -> String {
    let filled = vol / 10;
    let empty = 10 - filled;
    format!("{}{}", "█".repeat(filled as usize), "░".repeat(empty as usize))
}
