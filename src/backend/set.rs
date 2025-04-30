// backend/set.rs
// handles setting state in the backend

use std::process::{Command, Stdio};
use anyhow::Result;

use crate::models::{CurrentColumn, MPVSOCKET, RepeatMode, Track, Playlist};
use super::Backend;

// --- playback control ---

pub fn playpause(backend: &mut Backend) -> Result<()> {
    if backend.state.player.isplaying {
        pause(backend)?;
    } else {
        play(backend)?;
    }
    Ok(())
}

pub fn play(backend: &mut Backend) -> Result<()> {
    if !backend.state.player.queuestate.queue.is_empty() {
        backend.state.player.isplaying = true;
        // TODO: send play command to mpv
    }
    Ok(())
}

pub fn pause(backend: &mut Backend) -> Result<()> {
    backend.state.player.isplaying = false;
    // TODO: send pause command to mpv
    Ok(())
}

pub fn next(backend: &mut Backend) -> Result<()> {
    if !backend.state.player.queuestate.queue.is_empty() {
        // move current track to history
        let current = backend.state.player.queuestate.queue.remove(0);
        backend.state.player.queuestate.history.push(current);
        
        // play next track if available
        if !backend.state.player.queuestate.queue.is_empty() {
            playsong(backend)?;
        } else {
            // handle repeat mode
            handlerepeat(backend)?;
        }
    }
    Ok(())
}

pub fn prev(backend: &mut Backend) -> Result<()> {
    if !backend.state.player.queuestate.history.is_empty() {
        // get the last track from history
        let prev = backend.state.player.queuestate.history.pop().unwrap();
        
        // add current track back to the queue
        if !backend.state.player.queuestate.queue.is_empty() {
            let current = backend.state.player.queuestate.queue.remove(0);
            backend.state.player.queuestate.queue.insert(0, prev);
            backend.state.player.queuestate.queue.insert(1, current);
        } else {
            backend.state.player.queuestate.queue.insert(0, prev);
        }
        
        // play the track
        playsong(backend)?;
    }
    Ok(())
}

// --- playback settings ---

pub fn toggleshuffle(backend: &mut Backend) {
    backend.state.player.shufflestate.shuffle = !backend.state.player.shufflestate.shuffle;
    // TODO: implement shuffle logic
}

pub fn cyclerepeat(backend: &mut Backend) {
    backend.state.player.repeatstate.repeatmode = match backend.state.player.repeatstate.repeatmode {
        RepeatMode::None => RepeatMode::One,
        RepeatMode::One => RepeatMode::All,
        RepeatMode::All => RepeatMode::None
    };
}

fn handlerepeat(backend: &mut Backend) -> Result<()> {
    if backend.state.player.queuestate.queue.is_empty() {
        match backend.state.player.repeatstate.repeatmode {
            RepeatMode::None => {
                // do nothing
            },
            RepeatMode::One => {
                if let Some(lasttrack) = backend.state.player.queuestate.history.last().cloned() {
                    backend.state.player.queuestate.queue.insert(0, lasttrack);
                    playsong(backend)?;
                }
            },
            RepeatMode::All => {
                let mut history = Vec::new();
                std::mem::swap(&mut history, &mut backend.state.player.queuestate.history);
                history.reverse();
                backend.state.player.queuestate.queue = history;
                if !backend.state.player.queuestate.queue.is_empty() {
                    playsong(backend)?;
                }
            }
        }
    }
    Ok(())
}

// ---queue and playlist management ---

pub fn setqueue(backend: &mut Backend, tracks: Vec<Track>, clearhistory: bool) {
    backend.state.player.queuestate.queue = tracks;
    if clearhistory {
        backend.state.player.queuestate.history.clear();
    }
}

pub fn addtoqueue(backend: &mut Backend, track: Track) {
    backend.state.player.queuestate.queue.push(track);
}

pub fn playnext(backend: &mut Backend, track: Track) {
    backend.state.player.queuestate.queue.insert(0, track);
}

pub fn playtrackfromqueue(backend: &mut Backend, index: usize) -> Result<()> {
    if let Some(track) = backend.state.player.queuestate.queue.get(index).cloned() {
        backend.state.player.queuestate.history.push(track);
        backend.state.player.queuestate.queue.remove(index);
    }
    playsong(backend)?;
    Ok(())
}

pub fn addplaylist(backend: &mut Backend, playlist: Playlist) {
    backend.state.playlists.push(playlist);
}

pub fn selectplaylist(backend: &mut Backend, index: usize) -> Result<()> {
    if index < backend.state.playlists.len() {
        backend.selection.playliststate.select(Some(index));
    } else if !backend.state.playlists.is_empty() { 
        backend.selection.playliststate.select(Some(backend.state.playlists.len() - 1));
    } else {
        backend.selection.playliststate.select(None);
    }
    Ok(())
}

fn playsong(backend: &mut Backend) -> Result<()> {
    if backend.state.player.queuestate.queue.is_empty() {
        return Ok(());
    }
    
    let currenturl = &backend.state.player.queuestate.queue[0].url;
    
    if let Some(mut process) = backend.mpvprocess.take() {
        let _ = process.kill();
    }
    
    backend.mpvprocess = Some(Command::new("mpv")
        .arg(currenturl)
        .arg("--input-ipc-server=".to_string() + MPVSOCKET)
        .arg("--no-video")
        .arg("--no-terminal")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?);
    
    backend.state.player.isplaying = true;
    Ok(())
}

// --- navigation ---

pub fn nextcolumn(backend: &mut Backend) {
    backend.selection.selectedcolumn = match backend.selection.selectedcolumn {
        CurrentColumn::Playlists => CurrentColumn::Tracks,
        CurrentColumn::Tracks => CurrentColumn::Queue,
        CurrentColumn::Queue => CurrentColumn::Queue,
    };
}

pub fn prevcolumn(backend: &mut Backend) {
    backend.selection.selectedcolumn = match backend.selection.selectedcolumn {
        CurrentColumn::Playlists => CurrentColumn::Playlists,
        CurrentColumn::Tracks => CurrentColumn::Playlists,
        CurrentColumn::Queue => CurrentColumn::Tracks,
    };
}

pub fn nextrow(backend: &mut Backend) {
    match backend.selection.selectedcolumn {
        CurrentColumn::Playlists => {
            let playlistslen = backend.state.playlists.len();
            if playlistslen > 0 {
                let current = backend.selection.playliststate.selected().unwrap_or(0);
                let next = if current + 1 >= playlistslen {
                    playlistslen - 1
                } else {
                    current + 1
                };
                backend.selection.playliststate.select(Some(next));
            }
        },
        CurrentColumn::Tracks => {
            if let Some(playlistidx) = backend.selection.playliststate.selected() {
                if let Some(playlist) = backend.state.playlists.get(playlistidx) {
                    let trackslen = playlist.tracks.len();
                    if trackslen > 0 {
                        let current = backend.selection.trackstate.selected().unwrap_or(0);
                        let next = if current + 1 >= trackslen {
                            trackslen - 1
                        } else {
                            current + 1
                        };
                        backend.selection.trackstate.select(Some(next));
                    }
                }
            }
        },
        CurrentColumn::Queue => {
            let queuelen = backend.state.player.queuestate.queue.len();
            if queuelen > 0 {
                let current = backend.selection.queuestate.selected().unwrap_or(0);
                let next = if current + 1 >= queuelen {
                    queuelen - 1
                } else {
                    current + 1
                };
                backend.selection.queuestate.select(Some(next));
            }
        }
    }
}

pub fn prevrow(backend: &mut Backend) {
    match backend.selection.selectedcolumn {
        CurrentColumn::Playlists => {
            if !backend.state.playlists.is_empty() {
                let current = backend.selection.playliststate.selected().unwrap_or(0);
                let next = if current == 0 {
                    0
                } else {
                    current - 1
                };
                backend.selection.playliststate.select(Some(next));
            }
        },
        CurrentColumn::Tracks => {
            if let Some(playlistidx) = backend.selection.playliststate.selected() {
                if let Some(_) = backend.state.playlists.get(playlistidx) {
                    let current = backend.selection.trackstate.selected().unwrap_or(0);
                    let next = if current == 0 {
                        0
                    } else {
                        current - 1
                    };
                    backend.selection.trackstate.select(Some(next));
                }
            }
        },
        CurrentColumn::Queue => {
            if !backend.state.player.queuestate.queue.is_empty() {
                let current = backend.selection.queuestate.selected().unwrap_or(0);
                let next = if current == 0 {
                    0
                } else {
                    current - 1
                };
                backend.selection.queuestate.select(Some(next));
            }
        }
    }
}
