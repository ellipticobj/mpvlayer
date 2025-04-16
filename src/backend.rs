// backend.rs
// core logic and state management

use crate::models::*;
use anyhow::Result;
use std::process::Child;
use std::process::{Command, Stdio};

pub struct Backend {
    state: AppState,
    selection: SelectionState,
    mpvprocess: Option<Child>,
}

impl Backend {
    pub fn new() -> Self {
        // initialize the backend 
        Backend {
            state: AppState {
                playlists: Vec::new(),
                player: PlayerState {
                    isplaying: false,
                    currenttrack: None,
                    currenttime: 0,
                    queue: Vec::new(),
                    currentqueueidx: 0,
                    repeatmode: RepeatMode::None,
                    shuffle: false,
                },
            },
            selection: SelectionState {
                selectedcolumn: CurrentColumn::Playlists, // playlists column selected on startup
                selectedplaylist: None,
                selectedtrack: None,
            },
            mpvprocess: None,
        }
    }

    // --- playback control methods ---
    pub fn play(&mut self) -> Result<()> {
        // TODO: implement logic to start/resume playback with mpv
        // v if no track is selected, select the first in queue or do nothing
        // - update self.state.player.isplaying
        // - spawn or communicate with mpv process
        if self.selection.selectedtrack.is_none() {
            
        } else {
            
        }
        Ok(())
    }
    
    /// pauses playback
    /// sends a pause command to mpv using sockets 
    ///
    /// # arguments
    /// - none
    ///
    /// # returns
    /// - none
    pub fn pause(&mut self) -> Result<()> {
        let echo_output = Command::new("echo")
            .arg(r#"{"command": ["cycle", "pause"]}"#)
            .stdout(Stdio::piped())
            .spawn()?;

        if let Some(echoout) = echo_output.stdout {
            let socatout = Command::new("socat")
                .arg("-")
                .arg("/tmp/mpvlayer")
                .stdin(Stdio::from(echoout)) 
                .stdout(Stdio::null()) 
                .stderr(Stdio::piped()) 
                .output()?;

            if !socatout.status.success() {
                let stderr = String::from_utf8_lossy(&socatout.stderr);
                eprintln!("failed to send pause command to mpv: {}", stderr);
            }
        } else {
            eprintln!("failed to get stdout from echo command");
        }
        self.state.player.isplaying = false;
        Ok(())
    }

    pub fn nexttrack(&mut self) -> Result<()> {
        // TODO: move to next track in queue based on repeat/shuffle settings
        // - update self.state.player.currentqueueidx and currenttrack
        // - call play() if needed
        Ok(())
    }

    pub fn prevtrack(&mut self) -> Result<()> {
        // TODO: move to previous track in queue based on repeat settings
        // - update self.state.player.currentqueueidx and currenttrack
        // - call play() if needed
        Ok(())
    }

    pub fn toggleshuffle(&mut self) -> Result<()> {
        // TODO: toggle shuffle mode and reshuffle queue if needed
        // - update self.state.player.shuffle
        Ok(())
    }

    pub fn cyclerepeat(&mut self) -> Result<()> {
        // TODO: cycle through repeat modes (None -> All -> One -> None)
        // - update self.state.player.repeatmode
        Ok(())
    }

    // --- queue and playlist management ---
    pub fn setqueue(&mut self, tracks: Vec<Track>) {
        // TODO: set the playback queue
        // - update self.state.player.queue
        // - reset currentqueueidx if needed
    }

    pub fn addtoqueue(&mut self, track: Track) {
        // TODO: Add a track to the end of the queue
        // - append to self.state.player.queue
    }

    pub fn selecttrackfromqueue(&mut self, index: usize) -> Result<()> {
        // TODO: Set the current track based on queue index
        // - update self.state.player.currentqueueidx and currenttrack
        // - trigger playback
        Ok(())
    }

    pub fn addplaylist(&mut self, playlist: Playlist) {
        // TODO: Add a playlist to the list
        // - append to self.state.playlists
    }
    
    /// selects playlist
    ///
    /// # arguments
    /// - index (usize): index to select
    ///
    /// # returns
    /// - none
    pub fn selectplaylist(&mut self, index: usize) -> Result<()> {
        if index <= self.state.playlists.len() {
            self.selection.selectedplaylist = Some(index as u32);
        } else if !self.state.playlists.is_empty() { 
            self.selection.selectedplaylist = Some(self.state.playlists.len() as u32);
        } else {
            self.selection.selectedplaylist = None;
        }
        Ok(())
    }

    pub fn playsong(&mut self, playlistindex: usize, trackindex: Option<usize>) -> Result<()> {
        // TODO: check if index is within bounds
        // - set queue to playlist tracks
        // - select first track or specified track
        // - trigger playback
        Ok(())
    }

    // --- state access methods (for frontend to read) ---
    pub fn getstate(&self) -> &AppState {
        &self.state
    }

    pub fn getplayingstate(&self) -> &bool {
        &self.state.player.isplaying
    }

    pub fn getplayerstate(&self) -> &PlayerState {
        &self.state.player
    }

    pub fn getplaylists(&self) -> &Vec<Playlist> {
        &self.state.playlists
    }

    pub fn getrepeatstate(&self) -> &RepeatMode {
        &self.state.player.repeatmode
    }

    pub fn getshufflestate(&self) -> &bool {
        &self.state.player.shuffle
    }

    pub fn getcurrentsong(&self) -> &Option<Track> {
        &self.state.player.currenttrack
    }

    pub fn getqueue(&self) -> &Vec<Track> {
        &self.state.player.queue
    }

    // --- cleanup (called on app exit) ---
    pub fn shutdown(&mut self) -> Result<()> {
        // TODO: clean up resources
        // - kill mpv process if running
        // - any other cleanup
        Ok(())
    }
}
