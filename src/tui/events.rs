use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::tui::app::{App, Focus, Mode, View};

pub fn handle(app: &mut App) -> Result<()> {
    if !event::poll(std::time::Duration::from_millis(200))? { return Ok(()); }
    let ev = event::read()?;
    let key = match ev {
        Event::Key(k) if k.kind == KeyEventKind::Press => k,
        _ => return Ok(()),
    };

    // Input modes intercept most keys
    match app.mode {
        Mode::Command | Mode::Search | Mode::Filter => {
            match key.code {
                KeyCode::Esc => {
                    app.mode = Mode::Normal;
                    app.input.clear();
                }
                KeyCode::Enter => {
                    let text = std::mem::take(&mut app.input);
                    match app.mode {
                        Mode::Command => app.execute_command(&text),
                        Mode::Search => app.run_search(&text),
                        Mode::Filter => { /* future: filter tree */ }
                        _ => {}
                    }
                    app.mode = Mode::Normal;
                }
                KeyCode::Backspace => { app.input.pop(); }
                KeyCode::Char(c) => { app.input.push(c); }
                _ => {}
            }
            return Ok(());
        }
        Mode::Normal => {}
    }

    // Global keys
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.show_help_overlay { app.show_help_overlay = false; }
            else { app.quit = true; }
        }
        KeyCode::Char('?') => app.show_help_overlay = !app.show_help_overlay,
        KeyCode::Char(':') => { app.mode = Mode::Command; app.input.clear(); }
        KeyCode::Char('/') => { app.mode = Mode::Search; app.input.clear(); }
        KeyCode::Tab => {
            app.focus = match app.focus {
                Focus::Tree => Focus::Main,
                Focus::Main => Focus::Tree,
                Focus::CommandBar => Focus::Tree,
            };
        }
        KeyCode::Char('g') => { app.view = View::GitStatus; app.refresh_git_status(); }
        KeyCode::Char('x') => { app.view = View::Hex; }
        KeyCode::Char('H') => { app.view = View::Health; }
        KeyCode::Char('r') => {
            app.tree = crate::tui::app::build_tree(&app.root, 0, 2).unwrap_or_default();
            app.refresh_git_status();
            app.status_msg = "Refreshed.".to_string();
        }
        // Pane-specific navigation
        _ => match app.focus {
            Focus::Tree => handle_tree(app, key.code, key.modifiers),
            Focus::Main => handle_main(app, key.code),
            Focus::CommandBar => {}
        }
    }
    Ok(())
}

fn handle_tree(app: &mut App, code: KeyCode, _mods: KeyModifiers) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => app.move_tree(1),
        KeyCode::Up   | KeyCode::Char('k') => app.move_tree(-1),
        KeyCode::PageDown => app.move_tree(10),
        KeyCode::PageUp   => app.move_tree(-10),
        KeyCode::Right | KeyCode::Char('l') => {
            if let Some(e) = app.selected_entry() {
                if e.is_dir && !e.expanded { app.toggle_expand(); }
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if let Some(e) = app.selected_entry() {
                if e.is_dir && e.expanded { app.toggle_expand(); }
            }
        }
        KeyCode::Enter => app.open_selected(),
        _ => {}
    }
}

fn handle_main(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => app.scroll_preview(1),
        KeyCode::Up   | KeyCode::Char('k') => app.scroll_preview(-1),
        KeyCode::PageDown => app.scroll_preview(20),
        KeyCode::PageUp   => app.scroll_preview(-20),
        KeyCode::Home => app.preview_scroll = 0,
        KeyCode::End => app.preview_scroll = app.preview_lines.len().saturating_sub(1),
        KeyCode::Enter => {
            if app.view == View::Search {
                if let Some((p, ln, _)) = app.search_results.get(app.search_selected).cloned() {
                    app.load_preview(&p);
                    app.preview_scroll = ln.saturating_sub(1);
                }
            }
        }
        _ => {}
    }
}
