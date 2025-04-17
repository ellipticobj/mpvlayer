// backend.rs
// core logic and state management

use crate::models::*;
use anyhow::Result;
use std::process::Child;
use std::process::{Command, Stdio};
use crate::models::{PlayerState, QueueState};

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
                    currenttime: 0,
                    queuestate: QueueState {
                        queue: Vec::new(),
                        history: Vec::new(),
                    },
                    repeatmode: RepeatState {
                        repeatmode: RepeatMode::None,
                        originalqueue: vec![]
                    },
                    shuffle: ShuffleState {
                        shuffle: false,
                        originalqueue: vec![]
                    }
                }
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

    /// plays or pauses the current track depending on current state
    ///
    /// # arguments
    /// - none
    /// 
    /// # returns
    /// - none
    pub fn playpause(&mut self) -> Result<()> {
        if self.state.player.isplaying {
            self.pause()?;
        } else {
            self.unpause()?;
        }
        Ok(())
    }

    /// unpauses the current track in the queue
    /// 
    /// # arguments
    /// - none
    /// 
    /// # returns
    /// - none
    fn unpause(&mut self) -> Result<()> {
        let echooutput = Command::new("echo")
            .arg(r#"{"command": ["cycle", "pause"]}"#)
            .stdout(Stdio::piped())
            .spawn()?;

        if let Some(echoout) = echooutput.stdout {
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

    /// pauses playback
    /// sends a pause command to mpv using sockets 
    ///
    /// # arguments
    /// - none
    ///
    /// # returns
    /// - none
    fn pause(&mut self) -> Result<()> {
        let echooutput = Command::new("echo")
            .arg(r#"{"command": ["cycle", "pause"]}"#)
            .stdout(Stdio::piped())
            .spawn()?;
        
        if let Some(echoout) = echooutput.stdout {
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

    /// plays next track in queue
    ///
    /// # arguments
    /// - none
    ///
    /// # returns
    /// - none
    pub fn next(&mut self) -> Result<()> {
        if let Some(track) = self.state.player.queuestate.queue.get(0).cloned() {
            self.state.player.queuestate.history.push(track);
            self.state.player.queuestate.queue.remove(0);
            self.playsong()?;
        }
        Ok(())
    }

    /// plays previous track (last track in history)
    ///
    /// # arguments
    /// - none
    ///
    /// # returns
    /// - none
    pub fn prev(&mut self) -> Result<()> {
        if let Some(track) = self.state.player.queuestate.history.pop() {
            self.state.player.queuestate.queue.insert(0, track);
        }
        self.playsong()?;
        Ok(())
    }

    /// toggles shuffle
    ///
    /// # arguments
    /// - none
    ///
    /// # returns
    /// - none
    pub fn toggleshuffle(&mut self) {
        self.state.player.shufflestate.shuffle = !self.state.player.shufflestate.shuffle;
        self.shufflequeue();
    }

    fn shufflequeue(&mut self) {
        if self.state.player.shuffle && self.state.player.queuestate.queue.len() > 1 {
            
            let current = self.state.player.queuestate.queue.remove(0);
            self.state.player.queuestate.queue.shufflestate.shuffle(&mut rand::thread_rng());
            self.state.player.queuestate.queue.insert(0, current);
        }
    }

    /// cycles through repeat modes
    /// None -> One -> All -> None
    ///
    /// # arguments
    /// - none
    ///
    /// # returns
    /// - none
    pub fn cyclerepeat(&mut self) {
        self.state.player.repeatstate.repeatmode = match self.state.player.repeatstate.repeatmode {
            RepeatMode::None => RepeatMode::One,
            RepeatMode::One => RepeatMode::All,
            RepeatMode::All => RepeatMode::None
        };
    }

    // --- queue and playlist management ---

    /// sets the playback queue
    ///
    /// # arguments
    /// - tracks (Vec<Track>): tracks to set as queue
    /// - clearhistory (bool): whether to clear the playback history
    ///
    /// # returns
    /// - none
    pub fn setqueue(&mut self, tracks: Vec<Track>, clearhistory: bool) {
        self.state.player.queuestate.queue = tracks;
        if clearhistory {
            self.state.player.queuestate.history.clear();
        }
    }

    /// adds a song to the end of the queue
    ///
    /// # arguments
    /// - track (Track): track to add
    ///
    /// # returns
    /// - none
    pub fn addtoqueue(&mut self, track: Track) {
        self.state.player.queuestate.queue.push(track);
    }

    /// adds a song to the start of the queue
    ///
    /// # arguments
    /// - track (Track): track to add
    ///
    /// # returns
    /// - none
    pub fn playnext(&mut self, track: Track) {
        self.state.player.queuestate.queue.insert(0, track);
    }

    pub fn playtrackfromqueue(&mut self, index: usize) -> Result<()> {
        if let Some(track) = self.state.player.queuestate.queue.get(index).cloned() {
            self.state.player.queuestate.history.push(track);
            self.state.player.queuestate.queue.remove(index);
        }
        self.playsong()?;
        Ok(())
    }

    pub fn addplaylist(&mut self, playlist: Playlist) {
        self.state.playlists.push(playlist)
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
            self.selection.selectedplaylist = Some(index);
        } else if !self.state.playlists.is_empty() { 
            self.selection.selectedplaylist = Some(self.state.playlists.len());
        } else {
            self.selection.selectedplaylist = None;
        }
        Ok(())
    }

    fn playsong(&mut self) -> Result<()> {
        if self.state.player.queuestate.queue.is_empty() {
            return Ok(());
        }
        
        let currenturl = &self.state.player.queuestate.queue[0].url;
        
        if let Some(mut process) = self.mpvprocess.take() {
            let _ = process.kill();
        }
        
        self.mpvprocess = Some(Command::new("mpv")
            .arg(currenturl)
            .arg("--input-ipc-server=/tmp/mpvlayer")
            .arg("--no-video")
            .arg("--no-terminal")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?);
        
        self.state.player.isplaying = true;
        Ok(())
    }

    // --- navigation ---
    
    /// selects the next column, stops at queue
    /// playlists > tracks > queue > queue
    ///
    /// # arguments
    /// - none
    ///
    /// # returns
    /// - none
    pub fn nextcolumn(&mut self) {
        self.selection.selectedcolumn = match self.selection.selectedcolumn {
            CurrentColumn::Playlists => CurrentColumn::Tracks,
            CurrentColumn::Tracks => CurrentColumn::Queue,
            CurrentColumn::Queue => CurrentColumn::Queue,
        };
    }

    /// selects the previous column, stops at playlists
    /// playlists < playlists < tracks < queue
    ///
    /// # arguments
    /// - none
    ///
    /// # returns
    /// - none
    pub fn prevcolumn(&mut self) {
        self.selection.selectedcolumn = match self.selection.selectedcolumn {
            CurrentColumn::Playlists => CurrentColumn::Playlists,
            CurrentColumn::Tracks => CurrentColumn::Playlists,
            CurrentColumn::Queue => CurrentColumn::Tracks,
        };
    }

    /// selects the next row, stops at the last one
    pub fn nextrow(&mut self) {
        match self.selection.selectedcolumn {
            CurrentColumn::Playlists => {
                let current = self.selection.selectedplaylist.unwrap_or(0);
                let numberofplaylists = self.state.playlists.len();
                let next = if current + 1 >= numberofplaylists {
                    numberofplaylists // change to 0 to cycle
                } else {
                    current + 1
                };
                self.selection.selectedplaylist = Some(next);
            },
            CurrentColumn::Tracks => {
                if let Some(playlistidx) = self.selection.selectedplaylist {
                    if let Some(playlist) = self.state.playlists.get(playlistidx) {
                        let current = self.selection.selectedtrack.unwrap_or(0);
                        let numberoftracks = playlist.tracks.len();
                        let next = if current + 1 >= numberoftracks {
                            numberoftracks // change to 0 to cycle
                        } else {
                            current + 1
                        };
                        self.selection.selectedtrack = Some(next);
                    }
                }
            },
            CurrentColumn::Queue => {
                let current = self.selection.selectedtrack.unwrap_or(0);
                let queuelength = self.state.player.queuestate.queue.len();
                let next = if current >= queuelength - 1 {
                    queuelength // change to 0 to cycle
                } else {
                    current + 1
                };
                self.selection.selectedtrack = Some(next);
            }
        }
    }

    pub fn prevrow(&mut self) {
        match self.selection.selectedcolumn {
            CurrentColumn::Playlists => {
                let current = self.selection.selectedplaylist.unwrap_or(0);
                let next = if current <= 0 {
                     0
                } else {
                    current - 1
                };
                self.selection.selectedplaylist = Some(next);
            },
            CurrentColumn::Tracks => {
                if let Some(playlistidx) = self.selection.selectedplaylist {
                    if let Some(playlist) = self.state.playlists.get(playlistidx) {
                        let current = self.selection.selectedtrack.unwrap_or(0);
                        let next = if current <= 0 {
                             0
                        } else {
                            current - 1
                        };
                        self.selection.selectedtrack = Some(next);
                    }
                }
            },
            CurrentColumn::Queue => {
                let current = self.selection.selectedtrack.unwrap_or(0);
                let next = if current <= 0 {
                    0
                } else {
                    current - 1
                };
                self.selection.selectedtrack = Some(next);
            }
        }
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

    pub fn getcurrentsong(&self) -> Option<&Track> {
        self.state.player.queuestate.queue.get(0)
    }

    pub fn getqueue(&self) -> &Vec<Track> {
        &self.state.player.queuestate.queue
    }

    // --- cleanup (called on app exit) ---
    pub fn shutdown(&mut self) -> Result<()> {
        if let Some(mut process) = self.mpvprocess.take() {
            let _ = process.kill();
        }
        
        Ok(())
    }
}
