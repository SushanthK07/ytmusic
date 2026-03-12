use ratatui::style::{Modifier, Style};

use crate::config::Theme;

pub fn title(theme: &Theme) -> Style {
    Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
}

pub fn selected(theme: &Theme) -> Style {
    Style::default().bg(theme.highlight_bg).fg(theme.text)
}

pub fn active_border(theme: &Theme) -> Style {
    Style::default().fg(theme.border_active)
}

pub fn inactive_border(theme: &Theme) -> Style {
    Style::default().fg(theme.border)
}

pub fn dim(theme: &Theme) -> Style {
    Style::default().fg(theme.text_dim)
}

pub fn accent(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD)
}

pub fn secondary(theme: &Theme) -> Style {
    Style::default().fg(theme.text_secondary)
}
