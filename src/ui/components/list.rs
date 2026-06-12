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

/// One numbered track row: "▸  1. Artist — Title [mm:ss]".
/// The prefix follows the selection; the highlight style only applies
/// when the list is focused.
pub fn track_line(i: usize, t: &YTrack, selected: bool, focused: bool) -> Line<'static> {
    Line::from(format!(
        "{}{:>3}. {} — {} [{}]",
        row_prefix(selected),
        i + 1,
        t.artists_str(),
        t.title,
        t.duration_str()
    ))
    .style(row_style(selected && focused))
}
