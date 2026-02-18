use crate::app::{App, ConfirmAction, Mode};
use crate::git;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;

/// Returns true if the app should quit
pub fn handle_events(app: &mut App) -> Result<bool> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(false);
    }

    if let Event::Key(key) = event::read()? {
        match &app.mode.clone() {
            Mode::Normal => handle_normal(app, key.code, key.modifiers)?,
            Mode::Diff | Mode::Files => handle_scroll(app, key.code)?,
            Mode::Confirm(action) => handle_confirm(app, key.code, action.clone())?,
            Mode::NewStash => handle_new_stash(app, key.code)?,
            Mode::Message(_) => {
                // Any key dismisses the message
                app.mode = Mode::Normal;
            }
        }
    }

    Ok(false)
}

fn handle_normal(app: &mut App, key: KeyCode, _mods: KeyModifiers) -> Result<bool> {
    // If searching, intercept keys
    if app.searching {
        match key {
            KeyCode::Esc => {
                app.searching = false;
                app.search_query.clear();
                app.selected = 0;
            }
            KeyCode::Enter => {
                app.searching = false;
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.selected = 0;
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
                app.selected = 0;
            }
            _ => {}
        }
        return Ok(false);
    }

    match key {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(true),

        // Navigation
        KeyCode::Up | KeyCode::Char('k') => app.move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_down(),

        // View diff
        KeyCode::Enter | KeyCode::Char('d') => {
            if app.selected_stash().is_some() {
                app.load_diff()?;
                app.mode = Mode::Diff;
            }
        }

        // View files
        KeyCode::Char('f') => {
            if app.selected_stash().is_some() {
                app.load_files()?;
                app.mode = Mode::Files;
            }
        }

        // Apply (keep stash)
        KeyCode::Char('a') => {
            if app.selected_stash().is_some() {
                app.mode = Mode::Confirm(ConfirmAction::Apply);
            }
        }

        // Pop (apply + delete)
        KeyCode::Char('p') => {
            if app.selected_stash().is_some() {
                app.mode = Mode::Confirm(ConfirmAction::Pop);
            }
        }

        // Drop (delete)
        KeyCode::Char('x') | KeyCode::Delete => {
            if app.selected_stash().is_some() {
                app.mode = Mode::Confirm(ConfirmAction::Drop);
            }
        }

        // New stash
        KeyCode::Char('n') => {
            app.new_stash_input.clear();
            app.new_stash_untracked = false;
            app.mode = Mode::NewStash;
        }

        // Search
        KeyCode::Char('/') => {
            app.search_query.clear();
            app.searching = true;
            app.selected = 0;
        }

        // Clear search
        KeyCode::Char('c') => {
            app.search_query.clear();
            app.selected = 0;
        }

        _ => {}
    }

    Ok(false)
}

fn handle_scroll(app: &mut App, key: KeyCode) -> Result<bool> {
    match key {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = Mode::Normal;
        }
        KeyCode::Up | KeyCode::Char('k') => app.scroll_diff_up(),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_diff_down(),
        KeyCode::PageUp => {
            for _ in 0..20 {
                app.scroll_diff_up();
            }
        }
        KeyCode::PageDown => {
            for _ in 0..20 {
                app.scroll_diff_down();
            }
        }
        _ => {}
    }
    Ok(false)
}

fn handle_confirm(app: &mut App, key: KeyCode, action: ConfirmAction) -> Result<bool> {
    match key {
        KeyCode::Char('y') | KeyCode::Enter => {
            if let Some(stash) = app.selected_stash() {
                let stash_name = stash.name.clone();
                let result = match action {
                    ConfirmAction::Apply => git::apply_stash(&stash_name)
                        .map(|_| "Stash applied successfully.".to_string()),
                    ConfirmAction::Pop => git::pop_stash(&stash_name)
                        .map(|_| "Stash popped successfully.".to_string()),
                    ConfirmAction::Drop => {
                        git::drop_stash(&stash_name).map(|_| "Stash dropped.".to_string())
                    }
                };

                match result {
                    Ok(msg) => {
                        app.reload()?;
                        app.mode = Mode::Message(msg);
                    }
                    Err(e) => {
                        app.mode = Mode::Message(format!("Error: {e}"));
                    }
                }
            }
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        _ => {}
    }
    Ok(false)
}

fn handle_new_stash(app: &mut App, key: KeyCode) -> Result<bool> {
    match key {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        KeyCode::Enter => {
            let msg = app.new_stash_input.trim().to_string();
            if !msg.is_empty() {
                match git::push_stash(&msg, app.new_stash_untracked) {
                    Ok(()) => {
                        app.reload()?;
                        app.mode = Mode::Message(format!("Stash '{}' created.", msg));
                    }
                    Err(e) => {
                        app.mode = Mode::Message(format!("Error: {e}"));
                    }
                }
            }
        }
        KeyCode::Backspace => {
            app.new_stash_input.pop();
        }
        KeyCode::Char('u') if app.new_stash_input.is_empty() => {
            // toggle untracked when input is empty via Ctrl-u-like shortcut
            app.new_stash_untracked = !app.new_stash_untracked;
        }
        KeyCode::Char(c) => {
            app.new_stash_input.push(c);
        }
        KeyCode::Tab => {
            app.new_stash_untracked = !app.new_stash_untracked;
        }
        _ => {}
    }
    Ok(false)
}