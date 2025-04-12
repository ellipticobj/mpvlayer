use crate::models::*;
use crate::backend::Backend;

use crossterm::event::KeyCode;
use ratatui::Frame;

pub struct Frontend {
    backend: Backend,
    selectedplaylist: usize,
    selectedtrack: usize,
    // ... which column is focused, popup state, etc. ...
}

impl Frontend {
    pub fn new() -> Self {
        Frontend {
            backend: Backend::new(),
            selectedplaylist: 0,
            selectedtrack: 0,
            // ...
        }
    }

    pub fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(' ') => self.backend.play(),
            KeyCode::Char('p') => self.backend.pause(),
            KeyCode::Char('n') => self.backend.next(),
            KeyCode::Char('b') => self.backend.prev(),
            _ => { /* ... */ }
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let playerstate = self.backend.getplayerstate();
        let playlists = self.backend.getplaylists();
        // ... draw UI using player_state and playlists ...
    }
}

pub fn runfrontend(backend: &mut Backend) -> anyhow::Result<()> {

    Ok(())
}