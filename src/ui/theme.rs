use ratatui::style::{Color, Modifier, Style};

pub const ACCENT: Color = Color::Red;
#[allow(dead_code)]
pub const ACCENT_DIM: Color = Color::Indexed(88);
pub const TEXT: Color = Color::White;
pub const TEXT_DIM: Color = Color::DarkGray;
pub const TEXT_SECONDARY: Color = Color::Gray;
pub const BORDER: Color = Color::Indexed(236);
pub const BORDER_ACTIVE: Color = Color::Red;
pub const HIGHLIGHT_BG: Color = Color::Indexed(235);
pub const PLAYING_INDICATOR: Color = Color::Green;

pub fn title() -> Style {
    Style::default().fg(TEXT).add_modifier(Modifier::BOLD)
}

pub fn selected() -> Style {
    Style::default().bg(HIGHLIGHT_BG).fg(TEXT)
}

pub fn active_border() -> Style {
    Style::default().fg(BORDER_ACTIVE)
}

pub fn inactive_border() -> Style {
    Style::default().fg(BORDER)
}

pub fn dim() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn accent() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn secondary() -> Style {
    Style::default().fg(TEXT_SECONDARY)
}
