pub mod theme;

use crate::app::{App, LibraryItem, Mode, Panel, PlaylistMode, RepeatMode, SettingsSection};
use crate::config::{Theme, THEME_PRESETS};
use crate::player::PlaybackState;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, LineGauge, List, ListItem, ListState, Padding, Paragraph, Wrap,
};
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App, t: &Theme) {
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

    render_header(frame, outer[0], app, t);
    render_main(frame, outer[1], app, t);
    render_player_bar(frame, outer[2], app, t);
    render_status_bar(frame, outer[3], app, t);

    if app.show_playlist_picker {
        render_playlist_picker(frame, area, app, t);
    }

    if app.show_help {
        render_help_overlay(frame, area, t);
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let vol = app.player_status.volume;
    let shuffle_indicator = if app.shuffle { " [S]" } else { "" };
    let repeat_indicator = match app.repeat {
        RepeatMode::Off => "",
        RepeatMode::All => " [R]",
        RepeatMode::One => " [R1]",
    };

    let header = Line::from(vec![
        Span::styled(" ytmusic ", theme::accent(t)),
        Span::styled("│ ", theme::dim(t)),
        Span::styled(
            format!("Vol: {}%{}{}", vol, shuffle_indicator, repeat_indicator),
            theme::secondary(t),
        ),
    ]);

    frame.render_widget(Paragraph::new(header), area);
}

fn render_main(frame: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(30)])
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(4)])
        .split(main_layout[0]);

    render_library(frame, left[0], app, t);
    render_queue_panel(frame, left[1], app, t);
    render_content(frame, main_layout[1], app, t);
}

fn render_library(frame: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let border_style = if app.active_panel == Panel::Library {
        theme::active_border(t)
    } else {
        theme::inactive_border(t)
    };

    let block = Block::default()
        .title(" Library ")
        .title_style(theme::title(t))
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    let items: Vec<ListItem> = LibraryItem::ALL
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let marker = if i == app.library_cursor { ">" } else { " " };
            let style = if i == app.library_cursor {
                theme::selected(t)
            } else {
                Style::default().fg(t.text)
            };
            ListItem::new(format!(" {} {}", marker, item.label())).style(style)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.library_cursor));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_queue_panel(frame: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let border_style = if app.active_panel == Panel::Queue {
        theme::active_border(t)
    } else {
        theme::inactive_border(t)
    };

    let title = format!(" Queue ({}) ", app.queue.len());
    let block = Block::default()
        .title(title)
        .title_style(theme::title(t))
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    if app.queue.is_empty() {
        let empty = Paragraph::new(Span::styled("  Empty", theme::dim(t))).block(block);
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
                theme::selected(t)
            } else {
                Style::default().fg(t.text_dim)
            };
            let title = truncate(&track.title, (area.width as usize).saturating_sub(6));
            ListItem::new(format!(" {} {}", marker, title)).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let border_style = if app.active_panel == Panel::Content {
        theme::active_border(t)
    } else {
        theme::inactive_border(t)
    };

    match app.selected_library_item() {
        LibraryItem::Settings => {
            render_settings(frame, area, app, border_style, t);
            return;
        }
        LibraryItem::Favorites => {
            render_favorites(frame, area, app, border_style, t);
            return;
        }
        LibraryItem::Playlists => {
            render_playlists(frame, area, app, border_style, t);
            return;
        }
        _ => {}
    }

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
        render_search_input(frame, content_layout[0], app, t);
        render_search_results(frame, content_layout[1], app, border_style, t);
    } else {
        render_home(frame, content_layout[0], app, border_style, t);
    }
}

fn render_search_input(frame: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let border_style = if app.mode == Mode::Search {
        theme::active_border(t)
    } else {
        theme::inactive_border(t)
    };

    let block = Block::default()
        .title(" Search ")
        .title_style(theme::title(t))
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    let cursor_char = if app.mode == Mode::Search { "_" } else { "" };
    let display = format!(" {}{}", app.search_input, cursor_char);

    let input = Paragraph::new(display)
        .style(Style::default().fg(t.text))
        .block(block);

    frame.render_widget(input, area);
}

fn render_search_results(frame: &mut Frame, area: Rect, app: &App, border_style: Style, t: &Theme) {
    let title = if app.is_searching {
        " Searching... ".to_string()
    } else {
        format!(" Results ({}) ", app.search_results.len())
    };

    let block = Block::default()
        .title(title)
        .title_style(theme::title(t))
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    if app.search_results.is_empty() {
        let msg = if app.is_searching {
            "Searching..."
        } else {
            "No results"
        };
        let empty = Paragraph::new(Span::styled(format!("  {}", msg), theme::dim(t))).block(block);
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

    let now_playing_id = app.now_playing.as_ref().map(|tr| tr.video_id.as_str());

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
            let fav_icon = if app.is_favorited(&track.video_id) {
                "♥"
            } else {
                " "
            };

            let title_str = truncate(&track.title, title_col);
            let artist_str = truncate(&track.artist, artist_col);
            let duration = track.duration_text.as_deref().unwrap_or("--:--");

            let line = Line::from(vec![
                Span::raw(format!(" {} ", marker)),
                Span::styled(
                    format!("{:<width$}", title_str, width = title_col),
                    if is_playing {
                        Style::default()
                            .fg(t.playing_indicator)
                            .add_modifier(Modifier::BOLD)
                    } else if is_selected {
                        theme::selected(t)
                    } else {
                        Style::default().fg(t.text)
                    },
                ),
                Span::styled(
                    format!(" {:<width$}", artist_str, width = artist_col),
                    theme::secondary(t),
                ),
                Span::styled(format!(" {:>6} ", duration), theme::dim(t)),
                Span::styled(fav_icon, Style::default().fg(t.accent)),
                Span::styled(playing_icon, Style::default().fg(t.playing_indicator)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_home(frame: &mut Frame, area: Rect, _app: &App, border_style: Style, t: &Theme) {
    let block = Block::default()
        .title(" Home ")
        .title_style(theme::title(t))
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::new(2, 2, 1, 1));

    let welcome = vec![
        Line::from(vec![Span::styled(
            "Welcome to ytmusic",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press / to search for music",
            theme::secondary(t),
        )]),
        Line::from(vec![Span::styled(
            "Press ? for keyboard shortcuts",
            theme::secondary(t),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled("Requires: mpv, yt-dlp", theme::dim(t))]),
    ];

    let paragraph = Paragraph::new(welcome).block(block);
    frame.render_widget(paragraph, area);
}

fn render_favorites(frame: &mut Frame, area: Rect, app: &App, border_style: Style, t: &Theme) {
    let title = format!(" Favorites ({}) ", app.favorites_tracks.len());
    let block = Block::default()
        .title(title)
        .title_style(theme::title(t))
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    if app.favorites_tracks.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  No favorites yet — press f to favorite a track",
            theme::dim(t),
        ))
        .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let inner_width = area.width.saturating_sub(4) as usize;
    let visible_height = area.height.saturating_sub(2) as usize;
    let offset = scroll_offset(
        app.favorites_cursor,
        visible_height,
        app.favorites_tracks.len(),
    );

    let artist_col = inner_width.saturating_sub(8) / 3;
    let title_col = inner_width.saturating_sub(artist_col + 10);
    let now_playing_id = app.now_playing.as_ref().map(|tr| tr.video_id.as_str());

    let items: Vec<ListItem> = app
        .favorites_tracks
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible_height)
        .map(|(i, track)| {
            let is_selected = i == app.favorites_cursor;
            let is_playing = now_playing_id == Some(&track.video_id);
            let marker = if is_selected { ">" } else { " " };

            let line = Line::from(vec![
                Span::raw(format!(" {} ", marker)),
                Span::styled(
                    format!(
                        "{:<width$}",
                        truncate(&track.title, title_col),
                        width = title_col
                    ),
                    if is_playing {
                        Style::default()
                            .fg(t.playing_indicator)
                            .add_modifier(Modifier::BOLD)
                    } else if is_selected {
                        theme::selected(t)
                    } else {
                        Style::default().fg(t.text)
                    },
                ),
                Span::styled(
                    format!(
                        " {:<width$}",
                        truncate(&track.artist, artist_col),
                        width = artist_col
                    ),
                    theme::secondary(t),
                ),
                Span::styled(
                    format!(" {:>6} ", track.duration_text.as_deref().unwrap_or("--:--")),
                    theme::dim(t),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_playlists(frame: &mut Frame, area: Rect, app: &App, border_style: Style, t: &Theme) {
    match app.playlist_mode {
        PlaylistMode::Create => {
            let block = Block::default()
                .title(" New Playlist ")
                .title_style(theme::title(t))
                .borders(Borders::ALL)
                .border_style(border_style)
                .padding(Padding::new(2, 2, 1, 1));

            let lines = vec![
                Line::from(Span::styled("Enter playlist name:", theme::secondary(t))),
                Line::from(""),
                Line::from(Span::styled(
                    format!(" {}_", app.playlist_name_input),
                    Style::default().fg(t.text),
                )),
                Line::from(""),
                Line::from(Span::styled("Enter: create  Esc: cancel", theme::dim(t))),
            ];
            frame.render_widget(Paragraph::new(lines).block(block), area);
        }
        PlaylistMode::View => {
            let (title, tracks) = if let Some(idx) = app.viewing_playlist {
                if let Some(pl) = app.playlists.get(idx) {
                    (format!(" {} ({}) ", pl.name, pl.tracks.len()), &pl.tracks)
                } else {
                    (" Playlist ".to_string(), &Vec::new() as &Vec<_>)
                }
            } else {
                (" Playlist ".to_string(), &Vec::new() as &Vec<_>)
            };

            let block = Block::default()
                .title(title)
                .title_style(theme::title(t))
                .borders(Borders::ALL)
                .border_style(border_style)
                .padding(Padding::horizontal(1));

            if tracks.is_empty() {
                let empty =
                    Paragraph::new(Span::styled("  Empty playlist", theme::dim(t))).block(block);
                frame.render_widget(empty, area);
                return;
            }

            let inner_width = area.width.saturating_sub(4) as usize;
            let visible_height = area.height.saturating_sub(2) as usize;
            let offset = scroll_offset(app.playlist_track_cursor, visible_height, tracks.len());

            let artist_col = inner_width.saturating_sub(8) / 3;
            let title_col = inner_width.saturating_sub(artist_col + 10);

            let items: Vec<ListItem> = tracks
                .iter()
                .enumerate()
                .skip(offset)
                .take(visible_height)
                .map(|(i, track)| {
                    let is_selected = i == app.playlist_track_cursor;
                    let marker = if is_selected { ">" } else { " " };

                    let line = Line::from(vec![
                        Span::raw(format!(" {} ", marker)),
                        Span::styled(
                            format!(
                                "{:<width$}",
                                truncate(&track.title, title_col),
                                width = title_col
                            ),
                            if is_selected {
                                theme::selected(t)
                            } else {
                                Style::default().fg(t.text)
                            },
                        ),
                        Span::styled(
                            format!(
                                " {:<width$}",
                                truncate(&track.artist, artist_col),
                                width = artist_col
                            ),
                            theme::secondary(t),
                        ),
                        Span::styled(
                            format!(" {:>6} ", track.duration_text.as_deref().unwrap_or("--:--")),
                            theme::dim(t),
                        ),
                    ]);
                    ListItem::new(line)
                })
                .collect();

            let list = List::new(items).block(block);
            frame.render_widget(list, area);
        }
        PlaylistMode::List => {
            let title = format!(" Playlists ({}) ", app.playlists.len());
            let block = Block::default()
                .title(title)
                .title_style(theme::title(t))
                .borders(Borders::ALL)
                .border_style(border_style)
                .padding(Padding::horizontal(1));

            if app.playlists.is_empty() {
                let empty = Paragraph::new(Span::styled(
                    "  No playlists — press c to create one",
                    theme::dim(t),
                ))
                .block(block);
                frame.render_widget(empty, area);
                return;
            }

            let visible_height = area.height.saturating_sub(2) as usize;
            let offset = scroll_offset(app.playlist_cursor, visible_height, app.playlists.len());

            let items: Vec<ListItem> = app
                .playlists
                .iter()
                .enumerate()
                .skip(offset)
                .take(visible_height)
                .map(|(i, pl)| {
                    let is_selected = i == app.playlist_cursor;
                    let marker = if is_selected { ">" } else { " " };
                    let style = if is_selected {
                        theme::selected(t)
                    } else {
                        Style::default().fg(t.text)
                    };
                    ListItem::new(format!(
                        " {} {} ({} tracks)",
                        marker,
                        pl.name,
                        pl.tracks.len()
                    ))
                    .style(style)
                })
                .collect();

            let list = List::new(items).block(block);
            frame.render_widget(list, area);
        }
    }
}

fn render_settings(frame: &mut Frame, area: Rect, app: &App, border_style: Style, t: &Theme) {
    let block = Block::default()
        .title(" Settings ")
        .title_style(theme::title(t))
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::new(2, 2, 1, 1));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(THEME_PRESETS.len() as u16 + 3),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(inner);

    let theme_active = app.settings_section == SettingsSection::Theme;
    let theme_header_style = if theme_active {
        Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
    } else {
        theme::secondary(t)
    };

    let mut theme_lines = vec![
        Line::from(Span::styled("Theme", theme_header_style)),
        Line::from(""),
    ];

    for (i, &name) in THEME_PRESETS.iter().enumerate() {
        let is_current = name == app.current_theme_name;
        let is_cursor = theme_active && i == app.theme_cursor;

        let marker = if is_cursor { ">" } else { " " };
        let check = if is_current { " *" } else { "" };

        let style = if is_cursor {
            theme::selected(t)
        } else if is_current {
            Style::default().fg(t.accent)
        } else {
            Style::default().fg(t.text)
        };

        let label = format!("  {} {}{}", marker, name, check);
        theme_lines.push(Line::from(Span::styled(label, style)));
    }

    frame.render_widget(Paragraph::new(theme_lines), sections[0]);

    let vol_active = app.settings_section == SettingsSection::Volume;
    let vol_header_style = if vol_active {
        Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
    } else {
        theme::secondary(t)
    };

    let vol_lines = vec![
        Line::from(Span::styled("Default Volume", vol_header_style)),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {}%", app.player_status.volume),
                if vol_active {
                    theme::selected(t)
                } else {
                    Style::default().fg(t.text)
                },
            ),
            Span::styled(
                if vol_active { "  (-/+ to adjust)" } else { "" },
                theme::dim(t),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(vol_lines), sections[1]);

    let hint_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "j/k: navigate  Enter: apply  Tab: next section",
            theme::dim(t),
        )),
    ];
    frame.render_widget(Paragraph::new(hint_lines), sections[2]);
}

fn render_player_bar(frame: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let (icon, title_line) = match (&app.player_status.state, &app.now_playing) {
        (PlaybackState::Playing, Some(track)) => {
            ("▶", format!("{} — {}", track.title, track.artist))
        }
        (PlaybackState::Paused, Some(track)) => {
            ("⏸", format!("{} — {}", track.title, track.artist))
        }
        (PlaybackState::Buffering, Some(track)) => {
            ("⟳", format!("Loading: {} — {}", track.title, track.artist))
        }
        _ => ("■", "Not playing".to_string()),
    };

    let pos = app.player_status.position;
    let dur = app.player_status.duration;
    let time_str = format!("{} / {}", format_time(pos), format_time(dur));
    let title_width = area.width as usize - time_str.len() - 6;

    let now_playing = Line::from(vec![
        Span::styled(format!("  {} ", icon), theme::accent(t)),
        Span::styled(
            truncate(&title_line, title_width),
            Style::default().fg(t.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:>width$}  ", time_str, width = time_str.len()),
            theme::secondary(t),
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
        .filled_style(Style::default().fg(t.accent))
        .unfilled_style(Style::default().fg(t.border))
        .line_set(ratatui::symbols::line::THICK);

    let gauge_area = Rect {
        x: area.x + 2,
        width: area.width.saturating_sub(4),
        ..chunks[2]
    };
    frame.render_widget(gauge, gauge_area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let notification = app
        .notification
        .as_ref()
        .map(|(msg, _)| Span::styled(format!(" {} ", msg), Style::default().fg(t.accent)));

    let hints = if app.mode == Mode::Search {
        "enter:search  esc:cancel  ctrl+u:clear"
    } else {
        "space:play/pause  n/p:next/prev  /:search  a:queue  A:next  f:fav  P:playlist  ?:help  q:quit"
    };

    let bar = Line::from(vec![
        notification.unwrap_or(Span::raw("")),
        Span::styled(
            format!("{:>width$}", hints, width = area.width as usize),
            theme::dim(t),
        ),
    ]);

    frame.render_widget(Paragraph::new(bar), area);
}

fn render_playlist_picker(frame: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let track_name = app
        .playlist_picker_track
        .as_ref()
        .map(|tr| truncate(&tr.title, 30))
        .unwrap_or_default();

    let extra = if app.playlist_picker_creating { 2 } else { 0 };
    let list_height = (app.playlists.len() + 1).min(10) as u16;
    let popup_width = 44.min(area.width.saturating_sub(4));
    let popup_height = (list_height + 6 + extra as u16).min(area.height.saturating_sub(4));
    let popup = centered_rect(popup_width, popup_height, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Add to Playlist ")
        .title_style(theme::accent(t))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(theme::active_border(t))
        .padding(Padding::new(2, 2, 1, 1));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(inner);

    let header = Paragraph::new(vec![
        Line::from(Span::styled(track_name, Style::default().fg(t.text_dim))),
        Line::from(""),
    ]);
    frame.render_widget(header, chunks[0]);

    let new_pl_idx = app.playlists.len();
    let mut items: Vec<ListItem> = app
        .playlists
        .iter()
        .enumerate()
        .map(|(i, pl)| {
            let marker = if i == app.playlist_picker_cursor {
                ">"
            } else {
                " "
            };
            let style = if i == app.playlist_picker_cursor {
                theme::selected(t)
            } else {
                Style::default().fg(t.text)
            };
            ListItem::new(format!(
                " {} {} ({} tracks)",
                marker,
                pl.name,
                pl.tracks.len()
            ))
            .style(style)
        })
        .collect();

    let new_marker = if app.playlist_picker_cursor == new_pl_idx {
        ">"
    } else {
        " "
    };
    let new_style = if app.playlist_picker_cursor == new_pl_idx {
        theme::selected(t)
    } else {
        Style::default().fg(t.accent)
    };
    items.push(ListItem::new(format!(" {} + New Playlist", new_marker)).style(new_style));

    if app.playlist_picker_creating {
        items.push(
            ListItem::new(format!("     {}_", app.playlist_name_input))
                .style(Style::default().fg(t.text)),
        );
    }

    let list = List::new(items);
    frame.render_widget(list, chunks[1]);
}

fn render_help_overlay(frame: &mut Frame, area: Rect, t: &Theme) {
    let popup_width = 56.min(area.width.saturating_sub(4));
    let popup_height = 32.min(area.height.saturating_sub(4));

    let popup = centered_rect(popup_width, popup_height, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .title_style(theme::accent(t))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(theme::active_border(t))
        .padding(Padding::new(2, 2, 1, 1));

    let help_text = vec![
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  j/k, ↑/↓    ", theme::title(t)),
            Span::styled("Move cursor up/down", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  g/G          ", theme::title(t)),
            Span::styled("Go to top/bottom", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  Tab/h/l      ", theme::title(t)),
            Span::styled("Switch panels", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  Enter        ", theme::title(t)),
            Span::styled("Select / play", theme::secondary(t)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Playback",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Space        ", theme::title(t)),
            Span::styled("Play / pause", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  n/p          ", theme::title(t)),
            Span::styled("Next / previous track", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  >/<  ./, ", theme::title(t)),
            Span::styled("    Seek forward/backward 5s", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  +/-          ", theme::title(t)),
            Span::styled("Volume up/down", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  s            ", theme::title(t)),
            Span::styled("Toggle shuffle", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  r            ", theme::title(t)),
            Span::styled("Cycle repeat mode", theme::secondary(t)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Queue",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  a            ", theme::title(t)),
            Span::styled("Add to end of queue", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  A            ", theme::title(t)),
            Span::styled("Play next (insert at front)", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  d/x          ", theme::title(t)),
            Span::styled("Remove from queue", theme::secondary(t)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Library",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  f            ", theme::title(t)),
            Span::styled("Toggle favorite", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  P            ", theme::title(t)),
            Span::styled("Add to playlist (picker)", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  c            ", theme::title(t)),
            Span::styled("Create playlist (in Playlists)", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  Esc          ", theme::title(t)),
            Span::styled("Back to playlist list", theme::secondary(t)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  /            ", theme::title(t)),
            Span::styled("Search", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  ?            ", theme::title(t)),
            Span::styled("Toggle this help", theme::secondary(t)),
        ]),
        Line::from(vec![
            Span::styled("  q            ", theme::title(t)),
            Span::styled("Quit", theme::secondary(t)),
        ]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: false });
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
