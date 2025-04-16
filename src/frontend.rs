// frontend.rs
// handles drawing, etc

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::ListState,
    Terminal,
};
use std::io;
use std::time::Duration;

use crate::backend::Backend;
use crate::models::{CurrentColumn, RepeatMode, Track, Playlist};

pub fn runfrontend(backend: &mut Backend) -> Result<()> {
    // --- setup terminal ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backendtui = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backendtui)?;

    // --- initialize ui state ---
    // This is separate from backend state; it's just for UI-specific things
    let mut playliststate = ListState::default();
    let mut tracksstate = ListState::default();
    let mut queuestate = ListState::default();
    let mut currentcolumn = CurrentColumn::Playlists;
    let mut running = true;
    let version = String::from("0.0.1");

    // --- main loop ---
    let mut counter: u8 = 0;
    while running {
        // Draw the UI based on backend state
        let appstate = backend.getstate();
        terminal.draw(|frame| {
            // TODO: draw screen
        })?;

        // Handle input
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handleinput(backend, &mut running, &mut currentcolumn, &mut playliststate, &mut tracksstate, &mut queuestate, key.code)?;
                }
            }
        }

        // handle periodic updates (e.g., track progress)
        // TODO: Call backend methods to update player state
        counter = (counter + 1) % 4; // Example tick counter
        if counter == 0 {
            // TODO: Update current_time or check track end via backend
        }
    }

    // --- cleanup ---
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

fn handleinput(
    backend: &mut Backend,
    running: &mut bool,
    currentcolumn: &mut CurrentColumn,
    playliststate: &mut ListState,
    tracksstate: &mut ListState,
    queuestate: &mut ListState,
    key: KeyCode,
) -> Result<()> {
    // TODO: Handle popup if active (close on Enter/Esc)
    // if app.popup.onscreen { ... }

    // Handle input based on app state
    match key {
        KeyCode::Char('q') => *running = false,
        KeyCode::Char(' ') => backend.play()?, // toggle play/pause
        KeyCode::Char('>') => backend.next()?,
        KeyCode::Char('<') => {
            // TODO: restart song when < 10 secs
            backend.prev()?
        },
        KeyCode::Char('s') => backend.toggleshuffle(),
        KeyCode::Char('r') => backend.cyclerepeat(),
        // TODO: Navigation (Up/Down/Left/Right) to update UI state
        KeyCode::Up => { /* ... update selection ... */ }
        KeyCode::Enter => { /* ... select track or playlist ... */ }
        _ => {}
    }
    Ok(())
}
