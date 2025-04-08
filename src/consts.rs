use std::process::Child;

use ratatui::widgets::ListState;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub duration: u32,
    pub url: String
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<Track>
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RepeatType {
    None,
    One,
    All
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CurrentColumn {
    Playlists,
    Tracks,
    Queue
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub playing: bool,
    pub version: String,

    pub playlists: Vec<Playlist>,
    pub queue: Vec<Track>,
    pub queuebeforeshuffle: Option<Vec<Track>>,
    pub queuebeforerepeat: Option<Vec<Track>>,

    pub currentqueueidx: u32,
    pub currentplaylistidx: u32,
    pub currentdurationsecs: u32,

    pub shuffle: bool,
    pub repeat: RepeatType,

    pub mpv: Option<Child>,

    pub currentlyselectedplaylistidx: u32,
    pub currentlyselectedtrackidx: u32,
    pub currentlyselectedcolumn: CurrentColumn,

    pub playlistsstate: ListState,
    pub tracksstate: ListState,
    pub queuestate: ListState
}