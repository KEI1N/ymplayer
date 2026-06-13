use crate::api::models::YTrack;
use crate::ui::theme::Theme;
use ratatui::prelude::*;

pub fn row_prefix(selected: bool) -> &'static str {
    if selected { "▸ " } else { "  " }
}

/// Style for a list row; the highlighted row uses the theme's selection style.
pub fn row_style(highlighted: bool) -> Style {
    if highlighted {
        Theme::dark().selected
    } else {
        Style::default()
    }
}

/// Window [start, end) over `len` rows, `visible` rows tall, that keeps
/// `selected` roughly centered so the cursor never leaves the screen.
pub fn visible_window(len: usize, selected: usize, visible: usize) -> (usize, usize) {
    let visible = visible.max(1);
    let start = selected
        .saturating_sub(visible / 2)
        .min(len.saturating_sub(visible));
    let end = (start + visible).min(len);
    (start, end)
}

/// One numbered track row: "▸  1. ✓ Artist — Title [mm:ss]".
/// The ✓ marks a track already in the local cache. The prefix follows the
/// selection; the highlight style only applies when the list is focused.
pub fn track_line(i: usize, t: &YTrack, selected: bool, focused: bool, cached: bool) -> Line<'static> {
    let cache_mark = if cached {
        Span::styled("✓ ", Style::default().fg(Color::Green))
    } else {
        Span::raw("  ")
    };
    Line::from(vec![
        Span::raw(format!("{}{:>3}. ", row_prefix(selected), i + 1)),
        cache_mark,
        Span::raw(format!("{} — {} [{}]", t.artists_str(), t.title, t.duration_str())),
    ])
    .style(row_style(selected && focused))
}
