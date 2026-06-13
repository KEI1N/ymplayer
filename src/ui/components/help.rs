use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// Centered help overlay with the key bindings; drawn on top of any screen.
pub fn draw(frame: &mut Frame) {
    let screen = frame.size();
    let w = 62.min(screen.width.saturating_sub(2)).max(1);
    let h = 16.min(screen.height.saturating_sub(2)).max(1);
    let area = Rect::new(
        screen.x + (screen.width.saturating_sub(w)) / 2,
        screen.y + (screen.height.saturating_sub(h)) / 2,
        w,
        h,
    );

    let dim = Style::default().fg(Color::Rgb(120, 120, 120));
    let lines = vec![
        Line::from(" Навигация").style(dim),
        Line::from("   ↑/k ↓/j — курсор    ←/h →/l — панель/назад"),
        Line::from("   Tab / Shift+Tab — вкладки библиотеки"),
        Line::from("   PgUp/PgDn, Ctrl+u/d — на 10 позиций"),
        Line::from("   Enter — открыть / играть    Backspace — назад"),
        Line::from(" Воспроизведение").style(dim),
        Line::from("   Space — пауза    n / p — след. / пред. трек"),
        Line::from("   , / . — перемотка ±5 c    + / - — громкость"),
        Line::from("   s — перемешать    r — повтор    a — играть всё"),
        Line::from(" Экраны").style(dim),
        Line::from("   / — поиск    z — сейчас играет"),
        Line::from(" Прочее").style(dim),
        Line::from(vec![
            Span::raw("   "),
            Span::styled("✓", Style::default().fg(Color::Green)),
            Span::raw(" — трек в кэше    Ctrl+C — выход"),
        ]),
    ];

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Помощь (?/Esc — закрыть) ")),
        area,
    );
}
