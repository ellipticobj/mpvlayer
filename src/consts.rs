use std::process::Child;

pub struct Track {
    pub title: String,
    pub artist: String,
    pub duration: String,
    pub url: String
}

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
    pub currentqueueidx: u16,
    pub currentplaylist: Playlist,
    pub shuffle: bool,
    pub repeat: RepeatType,
    pub mpv: Option<Child>
}