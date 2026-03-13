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

pub fn render(frame: &mut Frame, app: &mut App, t: &Theme) {
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

    app.layout_areas.player_bar = outer[2];
    let gauge_area = Rect {
        x: outer[2].x + 2,
        width: outer[2].width.saturating_sub(4),
        y: outer[2].y + 2,
        height: 1,
    };
    app.layout_areas.progress_bar = gauge_area;

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

fn render_main(frame: &mut Frame, area: Rect, app: &mut App, t: &Theme) {
    let constraints = if app.show_lyrics {
        vec![
            Constraint::Length(20),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]
    } else {
        vec![Constraint::Length(20), Constraint::Min(30)]
    };

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(11), Constraint::Min(4)])
        .split(main_layout[0]);

    app.layout_areas.library = left[0];
    app.layout_areas.queue = left[1];
    app.layout_areas.content = main_layout[1];
    app.layout_areas.lyrics = if app.show_lyrics {
        Some(main_layout[2])
    } else {
        None
    };

    render_library(frame, left[0], app, t);
    render_queue_panel(frame, left[1], app, t);
    render_content(frame, main_layout[1], app, t);

    if app.show_lyrics {
        render_lyrics_pane(frame, main_layout[2], app, t);
    }
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
        LibraryItem::History => {
            render_history(frame, area, app, border_style, t);
            return;
        }
        LibraryItem::Explore => {
            render_explore(frame, area, app, border_style, t);
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

    let inner_width = area.width.saturating_sub(6) as usize;
    let visible_height = area.height.saturating_sub(2) as usize;
    let offset = scroll_offset(
        app.search_result_cursor,
        visible_height,
        app.search_results.len(),
    );

    let artist_col = inner_width.saturating_sub(16) / 3;
    let title_col = inner_width.saturating_sub(artist_col + 16);

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
                Span::styled(format!(" {:>6}", duration), theme::dim(t)),
                Span::styled(format!(" {}", fav_icon), Style::default().fg(t.accent)),
                Span::styled(
                    format!(" {} ", playing_icon),
                    Style::default().fg(t.playing_indicator),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_home(frame: &mut Frame, area: Rect, app: &App, border_style: Style, t: &Theme) {
    let block = Block::default()
        .title(" Home ")
        .title_style(theme::title(t))
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::new(2, 2, 1, 1));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let has_history = !app.history.is_empty();
    let history_height = if has_history {
        (app.history.len().min(8) + 3) as u16
    } else {
        0
    };

    let constraints = if has_history {
        vec![
            Constraint::Length(12),
            Constraint::Length(history_height),
            Constraint::Min(1),
        ]
    } else {
        vec![Constraint::Length(12), Constraint::Min(1)]
    };

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let accent_bold = Style::default().fg(t.accent).add_modifier(Modifier::BOLD);
    let key_style = Style::default().fg(t.accent);

    let mut lines = vec![
        Line::from(Span::styled("♫  ytmusic", accent_bold)),
        Line::from(Span::styled(
            "YouTube Music in your terminal",
            theme::dim(t),
        )),
        Line::from(""),
        Line::from(Span::styled("Quick Start", theme::secondary(t))),
        Line::from(""),
    ];

    let shortcuts = [
        ("/", "Search for music"),
        ("f", "Toggle favorite"),
        ("P", "Add to playlist"),
        ("L", "Show lyrics"),
        ("?", "All keyboard shortcuts"),
    ];

    for (key, desc) in &shortcuts {
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<4}", key), key_style),
            Span::styled(*desc, Style::default().fg(t.text)),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), sections[0]);

    if has_history {
        let mut history_lines = vec![
            Line::from(Span::styled("Recently Played", theme::secondary(t))),
            Line::from(""),
        ];

        let inner_width = inner.width.saturating_sub(2) as usize;
        let artist_col = inner_width.saturating_sub(6) / 3;
        let title_col = inner_width.saturating_sub(artist_col + 6);

        for entry in app.history.iter().rev().take(8) {
            history_lines.push(Line::from(vec![
                Span::styled("  ♫ ", Style::default().fg(t.accent_dim)),
                Span::styled(
                    format!(
                        "{:<width$}",
                        truncate(&entry.track.title, title_col),
                        width = title_col
                    ),
                    Style::default().fg(t.text),
                ),
                Span::styled(
                    format!(" {}", truncate(&entry.track.artist, artist_col)),
                    theme::secondary(t),
                ),
            ]));
        }

        frame.render_widget(Paragraph::new(history_lines), sections[1]);
    }

    let tip_idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| (d.as_secs() / 30) as usize)
        .unwrap_or(0);

    let tips = [
        "Press 'a' on a search result to add it to your queue",
        "Press 'A' to play a track next in your queue",
        "Press 's' to toggle shuffle, 'r' to cycle repeat modes",
        "Navigate panels with Tab or h/l arrow keys",
        "Press 'g' to jump to top, 'G' to jump to bottom",
        "Press 'n' for next track, 'p' for previous track",
        "Use +/- to adjust volume, >/<  to seek 5 seconds",
    ];

    let tip = tips[tip_idx % tips.len()];
    let tip_section = sections.last().unwrap();
    let tip_lines = vec![
        Line::from(""),
        Line::from(Span::styled(format!("  Tip: {}", tip), theme::dim(t))),
    ];
    frame.render_widget(Paragraph::new(tip_lines), *tip_section);
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

    let artist_col = inner_width.saturating_sub(12) / 3;
    let title_col = inner_width.saturating_sub(artist_col + 12);
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

            let artist_col = inner_width.saturating_sub(12) / 3;
            let title_col = inner_width.saturating_sub(artist_col + 12);

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

fn render_history(frame: &mut Frame, area: Rect, app: &App, border_style: Style, t: &Theme) {
    let title = format!(" History ({}) ", app.history.len());
    let block = Block::default()
        .title(title)
        .title_style(theme::title(t))
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    if app.history.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  No history yet — play a track to start",
            theme::dim(t),
        ))
        .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let inner_width = area.width.saturating_sub(4) as usize;
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.history.len();
    let offset = scroll_offset(app.history_cursor, visible_height, total);

    let time_col = 10;
    let artist_col = inner_width.saturating_sub(time_col + 12) / 3;
    let title_col = inner_width.saturating_sub(artist_col + time_col + 12);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let now_playing_id = app.now_playing.as_ref().map(|tr| tr.video_id.as_str());

    let items: Vec<ListItem> = (0..total)
        .rev()
        .enumerate()
        .skip(offset)
        .take(visible_height)
        .map(|(display_idx, rev_idx)| {
            let entry = &app.history[rev_idx];
            let is_selected = display_idx == app.history_cursor;
            let is_playing = now_playing_id == Some(&entry.track.video_id);
            let marker = if is_selected { ">" } else { " " };

            let ago = format_relative_time(now.saturating_sub(entry.played_at));

            let line = Line::from(vec![
                Span::raw(format!(" {} ", marker)),
                Span::styled(
                    format!(
                        "{:<width$}",
                        truncate(&entry.track.title, title_col),
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
                        truncate(&entry.track.artist, artist_col),
                        width = artist_col
                    ),
                    theme::secondary(t),
                ),
                Span::styled(format!(" {:>10} ", ago), theme::dim(t)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_explore(frame: &mut Frame, area: Rect, app: &App, border_style: Style, t: &Theme) {
    let block = Block::default()
        .title(" Explore ")
        .title_style(theme::title(t))
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    if app.explore_loading {
        let loading = Paragraph::new(Span::styled("  Loading...", theme::dim(t))).block(block);
        frame.render_widget(loading, area);
        return;
    }

    if app.explore_sections.is_empty() {
        let msg = if app.explore_loaded {
            "  No content available"
        } else {
            "  Loading explore..."
        };
        let empty = Paragraph::new(Span::styled(msg, theme::dim(t))).block(block);
        frame.render_widget(empty, area);
        return;
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let inner_width = inner.width.saturating_sub(2) as usize;
    let visible_height = inner.height as usize;
    let mut lines: Vec<Line> = Vec::new();

    for (si, section) in app.explore_sections.iter().enumerate() {
        let is_active_section = si == app.explore_section_cursor;
        let header_style = if is_active_section {
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            theme::secondary(t)
        };
        lines.push(Line::from(Span::styled(
            format!(" {}", section.title),
            header_style,
        )));
        lines.push(Line::from(""));

        for (ii, item) in section.items.iter().enumerate() {
            let is_selected = is_active_section && ii == app.explore_item_cursor;
            let marker = if is_selected { ">" } else { " " };

            let (label, detail) = match item {
                crate::api::BrowseItem::Track(track) => (
                    truncate(&track.title, inner_width / 2),
                    track.artist.clone(),
                ),
                crate::api::BrowseItem::PlaylistCard {
                    title, subtitle, ..
                } => (truncate(title, inner_width / 2), subtitle.clone()),
                crate::api::BrowseItem::Category { title, .. } => {
                    (truncate(title, inner_width / 2), String::new())
                }
            };

            let style = if is_selected {
                theme::selected(t)
            } else {
                Style::default().fg(t.text)
            };

            let title_w = inner_width / 2;
            let detail_w = inner_width.saturating_sub(title_w + 4);

            lines.push(Line::from(vec![
                Span::raw(format!(" {} ", marker)),
                Span::styled(format!("{:<width$}", label, width = title_w), style),
                Span::styled(
                    format!(" {}", truncate(&detail, detail_w)),
                    theme::secondary(t),
                ),
            ]));
        }

        lines.push(Line::from(""));
    }

    let total_lines = lines.len();
    let scroll = if total_lines > visible_height {
        let target = lines
            .iter()
            .take(total_lines)
            .enumerate()
            .filter(|(_, l)| {
                l.spans
                    .first()
                    .map(|s| s.content.starts_with(" >"))
                    .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .next()
            .unwrap_or(0);
        target.saturating_sub(visible_height / 3)
    } else {
        0
    };

    let visible_lines: Vec<Line> = lines
        .into_iter()
        .skip(scroll)
        .take(visible_height)
        .collect();
    frame.render_widget(Paragraph::new(visible_lines), inner);
}

fn format_relative_time(secs: u64) -> String {
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
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
            format!(
                "{:<width$}",
                truncate(&title_line, title_width),
                width = title_width
            ),
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
        "space:play/pause  n/p:next/prev  /:search  a:queue  A:next  f:fav  P:playlist  L:lyrics  ?:help  q:quit"
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

fn render_lyrics_pane(frame: &mut Frame, area: Rect, app: &App, t: &Theme) {
    let title = if app.lyrics_loading {
        " Lyrics (loading...) ".to_string()
    } else {
        " Lyrics ".to_string()
    };

    let block = Block::default()
        .title(title)
        .title_style(theme::title(t))
        .borders(Borders::ALL)
        .border_style(theme::active_border(t))
        .padding(Padding::new(1, 1, 1, 0));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.lyrics_lines.is_empty() && !app.lyrics_loading {
        let msg = if app.now_playing.is_some() {
            "No lyrics available"
        } else {
            "Play a track to see lyrics"
        };
        let empty = Paragraph::new(Span::styled(format!("  {}", msg), theme::dim(t)));
        frame.render_widget(empty, inner);
        return;
    }

    let visible = inner.height as usize;
    let current_idx = app.current_lyric_index();

    let scroll_to = if let Some(idx) = current_idx {
        idx.saturating_sub(visible / 3)
    } else {
        app.lyrics_scroll
    };

    let lines: Vec<Line> = app
        .lyrics_lines
        .iter()
        .enumerate()
        .skip(scroll_to)
        .take(visible)
        .map(|(i, text)| {
            let is_current = current_idx == Some(i);
            let style = if is_current {
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
            } else if text.is_empty() {
                Style::default()
            } else {
                Style::default().fg(t.text_dim)
            };
            Line::from(Span::styled(format!(" {}", text), style))
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
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
    let popup_height = 34.min(area.height.saturating_sub(4));

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
        Line::from(vec![
            Span::styled("  L            ", theme::title(t)),
            Span::styled("Toggle lyrics", theme::secondary(t)),
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
