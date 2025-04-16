// models.rs
// shared data structures for the music player app.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CurrentColumn {
    Playlists,
    Tracks,
    Queue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepeatMode {
    None,
    One,
    All,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueueState {
    pub queue: Vec<Track>,
    pub history: Vec<Track>
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub isplaying: bool,
    pub currenttime: u32, // in seconds
    pub queuestate: QueueState, 
    pub repeatmode: RepeatMode,
    pub shuffle: bool,
}

pub struct SelectionState {
    pub selectedcolumn: CurrentColumn,
    pub selectedplaylist: Option<usize>, 
    pub selectedtrack: Option<usize>
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub playlists: Vec<Playlist>,
    pub player: PlayerState,
}
