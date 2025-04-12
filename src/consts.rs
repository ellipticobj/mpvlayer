use std::process::Child;
use ratatui::widgets::ListState;
use std::fs::File;

pub static MAXQUEUELENGTH: usize = 50;
pub static MPVSOCKET: &str = "/tmp/mpvsocket";
pub static LOCKPATH: &str = "/tmp/mpvlayer.lock";

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
pub struct PopupState {
    pub onscreen: bool,
    pub dangerous: bool,
    pub title: String,
    pub message: Vec<String>
}

#[derive(Debug)]
pub struct App {
    pub running: bool,          // is the app running
    pub playing: bool,          // is music playing
    pub version: String,        // app version (x.x.x)
    pub repeatedinstance: bool, // stores if app is the second instance

    pub playlists: Vec<Playlist>,               // list of playlists
    pub queue: Vec<Track>,                      // queue of tracks
    pub queuebeforeshuffle: Option<Vec<Track>>, // queue before shuffle is active
    pub queuebeforerepeat: Option<Vec<Track>>,  // original queue

    pub currentqueueidx: u32,                   // index of currently playing track in teh queue
    pub currentplaylistidx: u32,                // index of playlist that the currently playing song is in
    pub currentdurationsecs: u32,               // elapsed duration in the currently playing track

    pub shuffle: bool,      // shuffle state
    pub repeat: RepeatType, // repeat state

    pub mpv: Option<Child>, // mpv process

    pub currentcolumn: CurrentColumn, // currently selected column (track, playlist, queue)
    pub playliststate: ListState,  // currently selecetd playlist
    pub tracksstate: ListState,     // currently selected track
    pub queuestate: ListState,      // currently selected track in queue

    pub lockfile: Option<File>,     // lock file for single instance check
    pub popup: PopupState,          // popup 
}

