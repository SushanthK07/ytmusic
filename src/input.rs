use crate::app::{App, LibraryItem, Mode, Panel, SettingsSection};
use crate::config::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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

    if app.in_settings() {
        return handle_settings(app, key).await;
    }

    let code = &key.code;
    let mods = &key.modifiers;
    let bindings = app.keybindings.clone();

    if bindings.matches(Action::Quit, code, mods) {
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
    } else if bindings.matches(Action::Select, code, mods) {
        if app.active_panel == Panel::Library {
            match app.selected_library_item() {
                LibraryItem::Search => app.enter_search(),
                LibraryItem::Queue => app.active_panel = Panel::Queue,
                LibraryItem::Home => app.active_panel = Panel::Content,
                LibraryItem::Settings => app.active_panel = Panel::Content,
            }
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
        app.remove_from_queue();
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
