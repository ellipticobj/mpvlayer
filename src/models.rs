pub static MAXQUEUELENGTH: usize = 50;
pub static MPVSOCKET: &str = "/tmp/mpvsocket";
pub static LOCKPATH: &str = "/tmp/mpvlayer.lock";

#[derive(Clone, Debug)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub duration: u32,
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug)]
pub enum RepeatMode {
    None,
    One,
    All,
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub playing: bool,
    pub currenttrack: Option<Track>,
    pub currenttrackidx: usize,
    pub currenttime: u32,
    pub queue: Vec<Track>,
    pub repeatmode: RepeatMode,
    pub shuffle: bool,
}