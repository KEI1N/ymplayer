use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub bg: Color,
    pub text: Color,
    pub text_dim: Color,
    pub highlight: Color,
    pub accent: Color,
    pub error: Color,
    pub success: Color,
    pub border: Color,
    pub title: Style,
    pub selected: Style,
    pub normal: Style,
    pub dim: Style,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg: Color::Reset,
            text: Color::Rgb(220, 220, 220),
            text_dim: Color::Rgb(120, 120, 120),
            highlight: Color::Rgb(60, 120, 200),
            accent: Color::Rgb(200, 100, 120),
            error: Color::Rgb(200, 60, 60),
            success: Color::Rgb(60, 200, 100),
            border: Color::Rgb(80, 80, 80),
            title: Style::default().fg(Color::Rgb(220, 220, 220)).add_modifier(Modifier::BOLD),
            selected: Style::default()
                .fg(Color::Rgb(220, 220, 220))
                .bg(Color::Rgb(60, 60, 120))
                .add_modifier(Modifier::BOLD),
            normal: Style::default().fg(Color::Rgb(200, 200, 200)),
            dim: Style::default().fg(Color::Rgb(100, 100, 100)),
        }
    }
}