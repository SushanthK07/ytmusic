use crate::app::{App, LibraryItem, Mode, Panel};
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

    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,

        KeyCode::Char('/') => app.enter_search(),
        KeyCode::Char('?') => app.show_help = true,

        KeyCode::Char('j') | KeyCode::Down => app.move_cursor_down(),
        KeyCode::Char('k') | KeyCode::Up => app.move_cursor_up(),
        KeyCode::Char('g') => app.move_cursor_top(),
        KeyCode::Char('G') => app.move_cursor_bottom(),

        KeyCode::Tab => app.next_panel(),
        KeyCode::BackTab => app.prev_panel(),
        KeyCode::Char('h') | KeyCode::Left => app.prev_panel(),
        KeyCode::Char('l') | KeyCode::Right => app.next_panel(),

        KeyCode::Enter => {
            if app.active_panel == Panel::Library {
                match app.selected_library_item() {
                    LibraryItem::Search => app.enter_search(),
                    LibraryItem::Queue => {
                        app.active_panel = Panel::Queue;
                    }
                    LibraryItem::Home => {
                        app.active_panel = Panel::Content;
                    }
                }
            } else {
                app.play_selected().await;
            }
        }

        KeyCode::Char(' ') => app.toggle_pause().await,
        KeyCode::Char('n') => app.play_next(),
        KeyCode::Char('p') => app.play_prev(),

        KeyCode::Char('>') | KeyCode::Char('.') => app.seek_forward().await,
        KeyCode::Char('<') | KeyCode::Char(',') => app.seek_backward().await,

        KeyCode::Char('+') | KeyCode::Char('=') => app.volume_up().await,
        KeyCode::Char('-') | KeyCode::Char('_') => app.volume_down().await,

        KeyCode::Char('s') => app.toggle_shuffle(),
        KeyCode::Char('r') => app.toggle_repeat(),

        KeyCode::Char('a') => app.add_to_queue(),
        KeyCode::Char('d') | KeyCode::Char('x') => app.remove_from_queue(),

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
