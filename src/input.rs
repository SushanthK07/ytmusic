use crate::app::{App, LibraryItem, Mode, Panel, PlaylistMode, SettingsSection};
use crate::config::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

pub async fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match app.mode {
        Mode::Normal => handle_normal(app, key).await,
        Mode::Search => handle_search(app, key).await,
    }
}

async fn handle_normal(app: &mut App, key: KeyEvent) -> bool {
    if app.show_help {
        app.show_help = false;
        return false;
    }

    if app.show_playlist_picker {
        return handle_playlist_picker(app, key);
    }

    if app.in_settings() {
        return handle_settings(app, key).await;
    }

    if app.active_panel == Panel::Content && app.selected_library_item() == LibraryItem::Playlists {
        return handle_playlists(app, key).await;
    }

    if app.active_panel == Panel::Content && app.selected_library_item() == LibraryItem::Explore {
        return handle_explore(app, key).await;
    }

    let code = &key.code;
    let mods = &key.modifiers;
    let bindings = app.keybindings.clone();

    if bindings.matches(Action::Quit, code, mods) {
        app.save_queue();
        return true;
    }

    if bindings.matches(Action::Search, code, mods) {
        app.enter_search();
    } else if bindings.matches(Action::Help, code, mods) {
        app.show_help = true;
    } else if bindings.matches(Action::MoveDown, code, mods) {
        app.move_cursor_down();
    } else if bindings.matches(Action::MoveUp, code, mods) {
        app.move_cursor_up();
    } else if bindings.matches(Action::MoveTop, code, mods) {
        app.move_cursor_top();
    } else if bindings.matches(Action::MoveBottom, code, mods) {
        app.move_cursor_bottom();
    } else if bindings.matches(Action::NextPanel, code, mods) {
        app.next_panel();
    } else if bindings.matches(Action::PrevPanel, code, mods) {
        app.prev_panel();
    } else if bindings.matches(Action::ToggleFavorite, code, mods) {
        app.toggle_favorite();
    } else if bindings.matches(Action::Select, code, mods) {
        if app.active_panel == Panel::Library {
            match app.selected_library_item() {
                LibraryItem::Search => app.enter_search(),
                LibraryItem::Queue => app.active_panel = Panel::Queue,
                LibraryItem::Home => app.active_panel = Panel::Content,
                LibraryItem::Settings => app.active_panel = Panel::Content,
                LibraryItem::Favorites => app.active_panel = Panel::Content,
                LibraryItem::History => {
                    app.active_panel = Panel::Content;
                    app.history_cursor = 0;
                }
                LibraryItem::Explore => {
                    app.active_panel = Panel::Content;
                    app.load_explore();
                }
                LibraryItem::Playlists => {
                    app.active_panel = Panel::Content;
                    app.playlist_mode = PlaylistMode::List;
                    app.viewing_playlist = None;
                    app.playlist_track_cursor = 0;
                }
            }
        } else if app.active_panel == Panel::Content
            && app.selected_library_item() == LibraryItem::Favorites
        {
            app.play_favorites().await;
        } else {
            app.play_selected().await;
        }
    } else if bindings.matches(Action::TogglePause, code, mods) {
        app.toggle_pause().await;
    } else if bindings.matches(Action::NextTrack, code, mods) {
        app.play_next();
    } else if bindings.matches(Action::PrevTrack, code, mods) {
        app.play_prev();
    } else if bindings.matches(Action::SeekForward, code, mods) {
        app.seek_forward().await;
    } else if bindings.matches(Action::SeekBackward, code, mods) {
        app.seek_backward().await;
    } else if bindings.matches(Action::VolumeUp, code, mods) {
        app.volume_up().await;
    } else if bindings.matches(Action::VolumeDown, code, mods) {
        app.volume_down().await;
    } else if bindings.matches(Action::ToggleShuffle, code, mods) {
        app.toggle_shuffle();
    } else if bindings.matches(Action::ToggleRepeat, code, mods) {
        app.toggle_repeat();
    } else if bindings.matches(Action::AddToQueue, code, mods) {
        app.add_to_queue();
    } else if bindings.matches(Action::PlayNext, code, mods) {
        app.play_next_in_queue();
    } else if bindings.matches(Action::RemoveFromQueue, code, mods) {
        if app.active_panel == Panel::Content
            && app.selected_library_item() == LibraryItem::Favorites
        {
            app.toggle_favorite();
        } else {
            app.remove_from_queue();
        }
    } else if bindings.matches(Action::AddToPlaylist, code, mods) {
        app.open_playlist_picker();
    } else if bindings.matches(Action::ToggleLyrics, code, mods) {
        app.toggle_lyrics();
    }

    false
}

fn handle_playlist_picker(app: &mut App, key: KeyEvent) -> bool {
    if app.playlist_picker_creating {
        match key.code {
            KeyCode::Enter => app.picker_create_playlist(),
            KeyCode::Esc => {
                app.playlist_picker_creating = false;
                app.playlist_name_input.clear();
                app.playlist_name_cursor = 0;
            }
            KeyCode::Char(c) => {
                app.playlist_name_input.insert(app.playlist_name_cursor, c);
                app.playlist_name_cursor += 1;
            }
            KeyCode::Backspace => {
                if app.playlist_name_cursor > 0 {
                    app.playlist_name_cursor -= 1;
                    app.playlist_name_input.remove(app.playlist_name_cursor);
                }
            }
            _ => {}
        }
        return false;
    }

    let max_cursor = app.playlists.len();
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.playlist_picker_cursor < max_cursor {
                app.playlist_picker_cursor += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.playlist_picker_cursor = app.playlist_picker_cursor.saturating_sub(1);
        }
        KeyCode::Enter => app.confirm_playlist_picker(),
        KeyCode::Esc | KeyCode::Char('q') => app.close_playlist_picker(),
        _ => {}
    }
    false
}

async fn handle_settings(app: &mut App, key: KeyEvent) -> bool {
    let code = &key.code;
    let mods = &key.modifiers;
    let bindings = app.keybindings.clone();

    if bindings.matches(Action::Quit, code, mods) {
        return true;
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => match app.settings_section {
            SettingsSection::Theme => app.settings_move_down(),
            SettingsSection::Volume => {}
        },
        KeyCode::Char('k') | KeyCode::Up => match app.settings_section {
            SettingsSection::Theme => app.settings_move_up(),
            SettingsSection::Volume => {}
        },
        KeyCode::Enter => app.settings_select(),
        KeyCode::Tab => app.settings_next_section(),
        KeyCode::Char('+') | KeyCode::Char('=') => {
            if app.settings_section == SettingsSection::Volume {
                app.settings_volume_up();
            }
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            if app.settings_section == SettingsSection::Volume {
                app.settings_volume_down();
            }
        }
        KeyCode::Char('h') | KeyCode::Left => app.prev_panel(),
        KeyCode::Char('l') | KeyCode::Right => app.next_panel(),
        KeyCode::Char('?') => app.show_help = true,
        KeyCode::Char(' ') => app.toggle_pause().await,
        _ => {}
    }

    false
}

async fn handle_search(app: &mut App, key: KeyEvent) -> bool {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('u') => {
                app.search_input.clear();
                app.search_cursor = 0;
            }
            KeyCode::Char('w') => {
                let input = &app.search_input[..app.search_cursor];
                let new_cursor = input.rfind(|c: char| c.is_whitespace()).unwrap_or(0);
                app.search_input.drain(new_cursor..app.search_cursor);
                app.search_cursor = new_cursor;
            }
            _ => {}
        }
        return false;
    }

    match key.code {
        KeyCode::Esc => app.exit_search(),
        KeyCode::Enter => app.submit_search(),
        KeyCode::Char(c) => {
            app.search_input.insert(app.search_cursor, c);
            app.search_cursor += 1;
        }
        KeyCode::Backspace => {
            if app.search_cursor > 0 {
                app.search_cursor -= 1;
                app.search_input.remove(app.search_cursor);
            }
        }
        KeyCode::Delete => {
            if app.search_cursor < app.search_input.len() {
                app.search_input.remove(app.search_cursor);
            }
        }
        KeyCode::Left => app.search_cursor = app.search_cursor.saturating_sub(1),
        KeyCode::Right => {
            if app.search_cursor < app.search_input.len() {
                app.search_cursor += 1;
            }
        }
        KeyCode::Home => app.search_cursor = 0,
        KeyCode::End => app.search_cursor = app.search_input.len(),
        _ => {}
    }

    false
}

async fn handle_explore(app: &mut App, key: KeyEvent) -> bool {
    let code = &key.code;
    let mods = &key.modifiers;
    let bindings = app.keybindings.clone();

    if bindings.matches(Action::Quit, code, mods) {
        app.save_queue();
        return true;
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.move_cursor_down(),
        KeyCode::Char('k') | KeyCode::Up => app.move_cursor_up(),
        KeyCode::Char('g') => app.move_cursor_top(),
        KeyCode::Char('G') => app.move_cursor_bottom(),
        KeyCode::Enter => app.play_selected().await,
        KeyCode::Esc => {
            if !app.explore_depth.is_empty() {
                app.browse_back();
            } else {
                app.prev_panel();
            }
        }
        KeyCode::Char('h') | KeyCode::Left => app.prev_panel(),
        KeyCode::Char('l') | KeyCode::Right => app.next_panel(),
        KeyCode::Char('/') => app.enter_search(),
        KeyCode::Char('f') => app.toggle_favorite(),
        KeyCode::Char(' ') => app.toggle_pause().await,
        KeyCode::Char('?') => app.show_help = true,
        _ => {}
    }

    false
}

async fn handle_playlists(app: &mut App, key: KeyEvent) -> bool {
    let code = &key.code;
    let mods = &key.modifiers;
    let bindings = app.keybindings.clone();

    if bindings.matches(Action::Quit, code, mods) {
        app.save_queue();
        return true;
    }

    match app.playlist_mode {
        PlaylistMode::Create => match key.code {
            KeyCode::Enter => app.create_playlist(),
            KeyCode::Esc => {
                app.playlist_name_input.clear();
                app.playlist_name_cursor = 0;
                app.playlist_mode = PlaylistMode::List;
            }
            KeyCode::Char(c) => {
                app.playlist_name_input.insert(app.playlist_name_cursor, c);
                app.playlist_name_cursor += 1;
            }
            KeyCode::Backspace => {
                if app.playlist_name_cursor > 0 {
                    app.playlist_name_cursor -= 1;
                    app.playlist_name_input.remove(app.playlist_name_cursor);
                }
            }
            _ => {}
        },
        PlaylistMode::View => match key.code {
            KeyCode::Char('j') | KeyCode::Down => app.move_cursor_down(),
            KeyCode::Char('k') | KeyCode::Up => app.move_cursor_up(),
            KeyCode::Char('g') => app.move_cursor_top(),
            KeyCode::Char('G') => app.move_cursor_bottom(),
            KeyCode::Enter => app.play_playlist_track().await,
            KeyCode::Esc => {
                app.playlist_mode = PlaylistMode::List;
                app.viewing_playlist = None;
                app.playlist_track_cursor = 0;
            }
            KeyCode::Char('d') | KeyCode::Char('x') => app.remove_from_playlist(),
            KeyCode::Char('f') => app.toggle_favorite(),
            KeyCode::Char('h') | KeyCode::Left => app.prev_panel(),
            KeyCode::Char('l') | KeyCode::Right => app.next_panel(),
            KeyCode::Char(' ') => app.toggle_pause().await,
            KeyCode::Char('?') => app.show_help = true,
            _ => {}
        },
        PlaylistMode::List => match key.code {
            KeyCode::Char('j') | KeyCode::Down => app.move_cursor_down(),
            KeyCode::Char('k') | KeyCode::Up => app.move_cursor_up(),
            KeyCode::Char('g') => app.move_cursor_top(),
            KeyCode::Char('G') => app.move_cursor_bottom(),
            KeyCode::Enter => {
                if !app.playlists.is_empty() {
                    app.viewing_playlist = Some(app.playlist_cursor);
                    app.playlist_track_cursor = 0;
                    app.playlist_mode = PlaylistMode::View;
                }
            }
            KeyCode::Char('c') => {
                app.playlist_mode = PlaylistMode::Create;
                app.playlist_name_input.clear();
                app.playlist_name_cursor = 0;
            }
            KeyCode::Char('d') => app.delete_playlist(),
            KeyCode::Char('h') | KeyCode::Left => app.prev_panel(),
            KeyCode::Char('l') | KeyCode::Right => app.next_panel(),
            KeyCode::Char('/') => app.enter_search(),
            KeyCode::Char('f') => app.toggle_favorite(),
            KeyCode::Char(' ') => app.toggle_pause().await,
            KeyCode::Char('?') => app.show_help = true,
            _ => {}
        },
    }

    false
}

pub async fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    let col = mouse.column;
    let row = mouse.row;

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let areas = app.layout_areas.clone();

            if contains(areas.progress_bar, col, row) {
                let bar_width = areas.progress_bar.width as f64;
                if bar_width > 0.0 && app.player_status.duration > 0.0 {
                    let click_x = (col - areas.progress_bar.x) as f64;
                    let ratio = click_x / bar_width;
                    let pos = ratio * app.player_status.duration;
                    app.seek_to(pos).await;
                }
                return;
            }

            if contains(areas.library, col, row) {
                app.active_panel = Panel::Library;
                let inner_y = (row - areas.library.y).saturating_sub(1) as usize;
                if inner_y < LibraryItem::ALL.len() {
                    app.library_cursor = inner_y;
                }
            } else if contains(areas.content, col, row) {
                app.active_panel = Panel::Content;
            } else if contains(areas.queue, col, row) {
                app.active_panel = Panel::Queue;
                let inner_y = (row - areas.queue.y).saturating_sub(1) as usize;
                if !app.queue.is_empty() && inner_y < app.queue.len() {
                    app.queue_cursor = inner_y;
                }
            }
        }
        MouseEventKind::ScrollDown => {
            let areas = app.layout_areas.clone();
            if contains(areas.content, col, row)
                || contains(areas.library, col, row)
                || contains(areas.queue, col, row)
            {
                app.move_cursor_down();
            }
        }
        MouseEventKind::ScrollUp => {
            let areas = app.layout_areas.clone();
            if contains(areas.content, col, row)
                || contains(areas.library, col, row)
                || contains(areas.queue, col, row)
            {
                app.move_cursor_up();
            }
        }
        _ => {}
    }
}

fn contains(rect: ratatui::layout::Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}
