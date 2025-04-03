use std::process::Child;

use ratatui::widgets::ListState;

#[derive(PartialEq, Eq, Clone)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub duration: String,
    pub url: String
}

#[derive(PartialEq, Eq, Clone)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<Track>
}

pub struct Queue {
    pub queue: Vec<Track>
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum RepeatType {
    None,
    One,
    All
}

pub struct App {
    pub running: bool,
    pub playing: bool,
    pub playlists: Vec<Playlist>,
    pub queue: Queue,
    pub currentqueueidx: u32,
    pub currentplaylistidx: u32,
    pub shuffle: bool,
    pub repeat: RepeatType,
    pub mpv: Option<Child>,
    pub currentlyselectedplaylistidx: u32,
    pub currentlyselectedtrackidx: u32,
    pub currentlyselectedplaylist: bool,
    pub playlistsstate: ListState,
    pub tracksstate: ListState
}