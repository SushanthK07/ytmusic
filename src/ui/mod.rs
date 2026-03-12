pub mod theme;

use crate::app::{App, LibraryItem, Mode, Panel, RepeatMode};
use crate::player::PlaybackState;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, LineGauge, List, ListItem, ListState, Padding, Paragraph, Wrap,
};
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(4),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(frame, outer[0], app);
    render_main(frame, outer[1], app);
    render_player_bar(frame, outer[2], app);
    render_status_bar(frame, outer[3], app);

    if app.show_help {
        render_help_overlay(frame, area);
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let vol = app.player_status.volume;
    let shuffle_indicator = if app.shuffle { " [S]" } else { "" };
    let repeat_indicator = match app.repeat {
        RepeatMode::Off => "",
        RepeatMode::All => " [R]",
        RepeatMode::One => " [R1]",
    };

    let header = Line::from(vec![
        Span::styled(" ytmusic ", theme::accent()),
        Span::styled("│ ", theme::dim()),
        Span::styled(
            format!("Vol: {}%{}{}", vol, shuffle_indicator, repeat_indicator),
            theme::secondary(),
        ),
    ]);

    frame.render_widget(Paragraph::new(header), area);
}

fn render_main(frame: &mut Frame, area: Rect, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(30)])
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(4)])
        .split(main_layout[0]);

    render_library(frame, left[0], app);
    render_queue_panel(frame, left[1], app);
    render_content(frame, main_layout[1], app);
}

fn render_library(frame: &mut Frame, area: Rect, app: &App) {
    let border_style = if app.active_panel == Panel::Library {
        theme::active_border()
    } else {
        theme::inactive_border()
    };

    let block = Block::default()
        .title(" Library ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    let items: Vec<ListItem> = LibraryItem::ALL
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let marker = if i == app.library_cursor { ">" } else { " " };
            let style = if i == app.library_cursor {
                theme::selected()
            } else {
                Style::default().fg(theme::TEXT)
            };
            ListItem::new(format!(" {} {}", marker, item.label())).style(style)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.library_cursor));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_queue_panel(frame: &mut Frame, area: Rect, app: &App) {
    let border_style = if app.active_panel == Panel::Queue {
        theme::active_border()
    } else {
        theme::inactive_border()
    };

    let title = format!(" Queue ({}) ", app.queue.len());
    let block = Block::default()
        .title(title)
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    if app.queue.is_empty() {
        let empty = Paragraph::new(Span::styled("  Empty", theme::dim())).block(block);
        frame.render_widget(empty, area);
        return;
    }

    let visible_height = area.height.saturating_sub(2) as usize;
    let offset = scroll_offset(app.queue_cursor, visible_height, app.queue.len());

    let items: Vec<ListItem> = app
        .queue
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible_height)
        .map(|(i, track)| {
            let marker = if i == app.queue_cursor { ">" } else { " " };
            let style = if i == app.queue_cursor {
                theme::selected()
            } else {
                Style::default().fg(theme::TEXT_DIM)
            };
            let title = truncate(&track.title, (area.width as usize).saturating_sub(6));
            ListItem::new(format!(" {} {}", marker, title)).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    let border_style = if app.active_panel == Panel::Content {
        theme::active_border()
    } else {
        theme::inactive_border()
    };

    let has_search_bar = app.mode == Mode::Search
        || !app.search_input.is_empty()
        || !app.search_results.is_empty()
        || app.is_searching;

    let constraints = if has_search_bar {
        vec![Constraint::Length(3), Constraint::Min(4)]
    } else {
        vec![Constraint::Min(4)]
    };

    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    if has_search_bar {
        render_search_input(frame, content_layout[0], app);
        render_search_results(frame, content_layout[1], app, border_style);
    } else {
        render_home(frame, content_layout[0], app, border_style);
    }
}

fn render_search_input(frame: &mut Frame, area: Rect, app: &App) {
    let border_style = if app.mode == Mode::Search {
        theme::active_border()
    } else {
        theme::inactive_border()
    };

    let block = Block::default()
        .title(" Search ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    let cursor_char = if app.mode == Mode::Search { "_" } else { "" };
    let display = format!(
        " {}{}",
        app.search_input,
        cursor_char
    );

    let input = Paragraph::new(display)
        .style(Style::default().fg(theme::TEXT))
        .block(block);

    frame.render_widget(input, area);
}

fn render_search_results(frame: &mut Frame, area: Rect, app: &App, border_style: Style) {
    let title = if app.is_searching {
        " Searching... ".to_string()
    } else {
        format!(" Results ({}) ", app.search_results.len())
    };

    let block = Block::default()
        .title(title)
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    if app.search_results.is_empty() {
        let msg = if app.is_searching {
            "Searching..."
        } else {
            "No results"
        };
        let empty = Paragraph::new(Span::styled(format!("  {}", msg), theme::dim())).block(block);
        frame.render_widget(empty, area);
        return;
    }

    let inner_width = area.width.saturating_sub(4) as usize;
    let visible_height = area.height.saturating_sub(2) as usize;
    let offset = scroll_offset(
        app.search_result_cursor,
        visible_height,
        app.search_results.len(),
    );

    let artist_col = inner_width.saturating_sub(8) / 3;
    let title_col = inner_width.saturating_sub(artist_col + 10);

    let now_playing_id = app.now_playing.as_ref().map(|t| t.video_id.as_str());

    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible_height)
        .map(|(i, track)| {
            let is_selected = i == app.search_result_cursor;
            let is_playing = now_playing_id == Some(&track.video_id);

            let marker = if is_selected { ">" } else { " " };
            let playing_icon = if is_playing { "♫" } else { " " };

            let title_str = truncate(&track.title, title_col);
            let artist_str = truncate(&track.artist, artist_col);
            let duration = track.duration_text.as_deref().unwrap_or("--:--");

            let line = Line::from(vec![
                Span::raw(format!(" {} ", marker)),
                Span::styled(
                    format!("{:<width$}", title_str, width = title_col),
                    if is_playing {
                        Style::default()
                            .fg(theme::PLAYING_INDICATOR)
                            .add_modifier(Modifier::BOLD)
                    } else if is_selected {
                        theme::selected()
                    } else {
                        Style::default().fg(theme::TEXT)
                    },
                ),
                Span::styled(
                    format!(" {:<width$}", artist_str, width = artist_col),
                    theme::secondary(),
                ),
                Span::styled(format!(" {:>6} ", duration), theme::dim()),
                Span::styled(
                    playing_icon,
                    Style::default().fg(theme::PLAYING_INDICATOR),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_home(frame: &mut Frame, area: Rect, _app: &App, border_style: Style) {
    let block = Block::default()
        .title(" Home ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::new(2, 2, 1, 1));

    let welcome = vec![
        Line::from(vec![Span::styled(
            "Welcome to ytmusic",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press / to search for music",
            theme::secondary(),
        )]),
        Line::from(vec![Span::styled(
            "Press ? for keyboard shortcuts",
            theme::secondary(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Requires: mpv, yt-dlp",
            theme::dim(),
        )]),
    ];

    let paragraph = Paragraph::new(welcome).block(block);
    frame.render_widget(paragraph, area);
}

fn render_player_bar(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let (icon, title_line) = match (&app.player_status.state, &app.now_playing) {
        (PlaybackState::Playing, Some(track)) => (
            "▶",
            format!("{} — {}", track.title, track.artist),
        ),
        (PlaybackState::Paused, Some(track)) => (
            "⏸",
            format!("{} — {}", track.title, track.artist),
        ),
        (PlaybackState::Buffering, Some(track)) => (
            "⟳",
            format!("Loading: {} — {}", track.title, track.artist),
        ),
        _ => ("■", "Not playing".to_string()),
    };

    let pos = app.player_status.position;
    let dur = app.player_status.duration;
    let time_str = format!("{} / {}", format_time(pos), format_time(dur));
    let title_width = area.width as usize - time_str.len() - 6;

    let now_playing = Line::from(vec![
        Span::styled(format!("  {} ", icon), theme::accent()),
        Span::styled(
            truncate(&title_line, title_width),
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:>width$}  ", time_str, width = time_str.len()),
            theme::secondary(),
        ),
    ]);

    frame.render_widget(Paragraph::new(now_playing), chunks[1]);

    let ratio = if dur > 0.0 {
        (pos / dur).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let gauge = LineGauge::default()
        .ratio(ratio)
        .filled_style(Style::default().fg(theme::ACCENT))
        .unfilled_style(Style::default().fg(theme::BORDER))
        .line_set(ratatui::symbols::line::THICK);

    let gauge_area = Rect {
        x: area.x + 2,
        width: area.width.saturating_sub(4),
        ..chunks[2]
    };
    frame.render_widget(gauge, gauge_area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let notification = app
        .notification
        .as_ref()
        .map(|(msg, _)| Span::styled(format!(" {} ", msg), Style::default().fg(theme::ACCENT)));

    let hints = if app.mode == Mode::Search {
        "enter:search  esc:cancel  ctrl+u:clear"
    } else {
        "space:play/pause  n/p:next/prev  /:search  a:queue  ?:help  q:quit"
    };

    let bar = Line::from(vec![
        notification.unwrap_or(Span::raw("")),
        Span::styled(
            format!("{:>width$}", hints, width = area.width as usize),
            theme::dim(),
        ),
    ]);

    frame.render_widget(Paragraph::new(bar), area);
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let popup_width = 56.min(area.width.saturating_sub(4));
    let popup_height = 24.min(area.height.saturating_sub(4));

    let popup = centered_rect(popup_width, popup_height, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .title_style(theme::accent())
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(theme::active_border())
        .padding(Padding::new(2, 2, 1, 1));

    let help_text = vec![
        Line::from(vec![
            Span::styled("Navigation", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  j/k, ↑/↓    ", theme::title()),
            Span::styled("Move cursor up/down", theme::secondary()),
        ]),
        Line::from(vec![
            Span::styled("  g/G          ", theme::title()),
            Span::styled("Go to top/bottom", theme::secondary()),
        ]),
        Line::from(vec![
            Span::styled("  Tab/h/l      ", theme::title()),
            Span::styled("Switch panels", theme::secondary()),
        ]),
        Line::from(vec![
            Span::styled("  Enter        ", theme::title()),
            Span::styled("Select / play", theme::secondary()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Playback", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Space        ", theme::title()),
            Span::styled("Play / pause", theme::secondary()),
        ]),
        Line::from(vec![
            Span::styled("  n/p          ", theme::title()),
            Span::styled("Next / previous track", theme::secondary()),
        ]),
        Line::from(vec![
            Span::styled("  >/<  ./, ", theme::title()),
            Span::styled("    Seek forward/backward 5s", theme::secondary()),
        ]),
        Line::from(vec![
            Span::styled("  +/-          ", theme::title()),
            Span::styled("Volume up/down", theme::secondary()),
        ]),
        Line::from(vec![
            Span::styled("  s            ", theme::title()),
            Span::styled("Toggle shuffle", theme::secondary()),
        ]),
        Line::from(vec![
            Span::styled("  r            ", theme::title()),
            Span::styled("Cycle repeat mode", theme::secondary()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Queue", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  a            ", theme::title()),
            Span::styled("Add selected to queue", theme::secondary()),
        ]),
        Line::from(vec![
            Span::styled("  d/x          ", theme::title()),
            Span::styled("Remove from queue", theme::secondary()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  /            ", theme::title()),
            Span::styled("Search", theme::secondary()),
        ]),
        Line::from(vec![
            Span::styled("  ?            ", theme::title()),
            Span::styled("Toggle this help", theme::secondary()),
        ]),
        Line::from(vec![
            Span::styled("  q            ", theme::title()),
            Span::styled("Quit", theme::secondary()),
        ]),
    ];

    let paragraph = Paragraph::new(help_text).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, popup);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

fn scroll_offset(cursor: usize, visible: usize, total: usize) -> usize {
    if total <= visible {
        return 0;
    }
    if cursor < visible / 2 {
        return 0;
    }
    let max_offset = total.saturating_sub(visible);
    (cursor.saturating_sub(visible / 2)).min(max_offset)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max > 3 {
        format!("{}...", &s[..max - 3])
    } else {
        s[..max].to_string()
    }
}

fn format_time(seconds: f64) -> String {
    if seconds <= 0.0 || seconds.is_nan() {
        return "--:--".to_string();
    }
    let total = seconds as u64;
    let mins = total / 60;
    let secs = total % 60;
    if mins >= 60 {
        let hours = mins / 60;
        format!("{}:{:02}:{:02}", hours, mins % 60, secs)
    } else {
        format!("{}:{:02}", mins, secs)
    }
}
