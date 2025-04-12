use crate::models::*;

pub struct Backend {
    pub playlists: Vec<Playlist>,
    pub player: PlayerState,
    // ...
}

impl Backend {
    pub fn new() -> Self {
        // load playlists/initialize player state, etc.
        Backend {
            playlists: vec![], // load/create default playlists
            player: PlayerState {
                playing: false,
                currenttrack: None,
                currenttrackidx: 0,
                currenttime: 0,
                queue: vec![],
                repeatmode: RepeatMode::None,
                shuffle: false,
            },
        }
    }

    // --- playback controls ---
    pub fn play(&mut self) { /* ... */ }
    pub fn pause(&mut self) { /* ... */ }
    pub fn next(&mut self) { /* ... */ }
    pub fn prev(&mut self) { /* ... */ }
    pub fn seek(&mut self, seconds: u32) { /* ... */ }

    // --- queue/playlist management ---
    pub fn addtoqueue(&mut self, track: Track) { /* ... */ }
    pub fn setqueue(&mut self, tracks: Vec<Track>) { /* ... */ }
    pub fn setplaylist(&mut self, playlist: &Playlist) { /* ... */ }

    // --- state access ---
    pub fn getplayerstate(&self) -> &PlayerState {
        &self.player
    }
    pub fn getplaylists(&self) -> &Vec<Playlist> {
        &self.playlists
    }
}
