// backend/set.rs
// handles setting state in the backend

use anyhow::{anyhow, Result};
use std::process::{Command, Stdio};

use super::Backend;
use crate::models::{CurrentColumn, Playlist, RepeatMode, Track, MPVSOCKET};

// --- helper functions ---
fn sendcommand(commandjson: &str) -> Result<()> {
    // only attempt to send a command if the MPV socket exists
    if !std::path::Path::new(MPVSOCKET).exists() {
        return Ok(());
    }

    // Use a consistent timeout
    let timeout = 3; // 3 seconds timeout, increased from 1 second

    // For each attempt, we need to create a new echo command
    // since we can only use the stdout pipe once

    // Allow for a few retries in case of temporary connection issues
    let max_retries = 2;
    let mut attempt = 0;
    let mut last_stderr = String::new();

    while attempt < max_retries {
        // Create a new echo command for each attempt
        let echochild = Command::new("echo")
            .arg(commandjson)
            .stdout(Stdio::piped())
            .spawn()?;

        if let Some(echostdout) = echochild.stdout {
            let socatout = Command::new("socat")
                .arg("-T")
                .arg(timeout.to_string())
                .arg("-")
                .arg(MPVSOCKET)
                .stdin(Stdio::from(echostdout))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;

            if socatout.status.success() {
                return Ok(());
            }

            last_stderr = String::from_utf8_lossy(&socatout.stderr).to_string();

            // Don't retry for these specific error cases
            if !last_stderr.contains("no such file or directory")
                && !last_stderr.contains("connection refused")
            {
                // For other errors, may be worth retrying
                attempt += 1;
                // Short pause before retry
                std::thread::sleep(std::time::Duration::from_millis(200));
            } else {
                // Socket not ready errors - break immediately
                break;
            }
        } else {
            return Err(anyhow!("failed to get stdout from echo for mpv command"));
        }
    }

    // If we get here, all attempts failed
    // But for socket-not-ready errors, just return Ok
    if last_stderr.contains("no such file or directory")
        || last_stderr.contains("connection refused")
    {
        // Socket isn't ready yet, but that's okay
        return Ok(());
    }

    // For other errors, report them
    Err(anyhow::anyhow!(
        "socat command failed for mpv: {}\ncommand: {}",
        last_stderr,
        commandjson
    ))
}

// --- playback control ---
pub fn playpause(backend: &mut Backend) -> Result<()> {
    if backend.state.player.queuestate.queue.is_empty()
        && backend.state.player.queuestate.history.is_empty()
    {
        return Err(anyhow::anyhow!("No tracks in queue to play"));
    }

    if backend.state.player.isplaying {
        pause(backend)?;
    } else {
        play(backend)?;
    }
    Ok(())
}

pub fn play(backend: &mut Backend) -> Result<()> {
    if !backend.state.player.queuestate.queue.is_empty() {
        if backend.mpvprocess.is_none() {
            // mpv not running, start it
            playsong(backend)?;
        } else {
            // mpv is running but was paused
            sendcommand(r#"{"command":["set_property","pause",false]}"#)?;
            backend.state.player.isplaying = true;
        }
    } else if backend.state.player.isplaying {
        // queue is empty but state says playing
        backend.state.player.isplaying = false;
    }

    Ok(())
}

pub fn pause(backend: &mut Backend) -> Result<()> {
    if backend.mpvprocess.is_some() && backend.state.player.isplaying {
        sendcommand(r#"{"command":["set_property","pause",true]}"#)?;
        backend.state.player.isplaying = false;
    }
    Ok(())
}

pub fn next(backend: &mut Backend) -> Result<()> {
    if backend.state.player.queuestate.queue.is_empty() {
        return Err(anyhow::anyhow!("No more tracks in queue"));
    }

    // move current track to history
    let current = backend.state.player.queuestate.queue.remove(0);
    backend.state.player.queuestate.history.push(current);

    // play next track if available
    if !backend.state.player.queuestate.queue.is_empty() {
        playsong(backend)?;
    } else {
        // handle repeat mode
        handlerepeat(backend)?;

        // if queue is still empty after repeat handling, throw error
        if backend.state.player.queuestate.queue.is_empty() {
            return Err(anyhow::anyhow!("end of queue reached"));
        }
    }

    Ok(())
}

pub fn prev(backend: &mut Backend) -> Result<()> {
    if backend.state.player.queuestate.history.is_empty() {
        return Err(anyhow::anyhow!("no previous tracks available"));
    }

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

    Ok(())
}

// --- playback settings ---

pub fn toggleshuffle(backend: &mut Backend) {
    backend.state.player.shufflestate.shuffle = !backend.state.player.shufflestate.shuffle;
    // TODO: implement shuffle logic
}

pub fn cyclerepeat(backend: &mut Backend) {
    backend.state.player.repeatstate.repeatmode = match backend.state.player.repeatstate.repeatmode
    {
        RepeatMode::None => RepeatMode::One,
        RepeatMode::One => RepeatMode::All,
        RepeatMode::All => RepeatMode::None,
    };
}

fn handlerepeat(backend: &mut Backend) -> Result<()> {
    if backend.state.player.queuestate.queue.is_empty() {
        match backend.state.player.repeatstate.repeatmode {
            RepeatMode::None => {
                // do nothing
            }
            RepeatMode::One => {
                if let Some(lasttrack) = backend.state.player.queuestate.history.last().cloned() {
                    backend.state.player.queuestate.queue.insert(0, lasttrack);
                    playsong(backend)?;
                }
            }
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

pub fn playtrackfromqueue(backend: &mut Backend, index: usize) -> Result<()> {
    if index >= backend.state.player.queuestate.queue.len() {
        return Err(anyhow::anyhow!("Invalid track index"));
    }

    // move current track to history if one is playing
    if !backend.state.player.queuestate.queue.is_empty() && backend.state.player.isplaying {
        let current = backend.state.player.queuestate.queue.remove(0);
        backend.state.player.queuestate.history.push(current);
    }

    // get the requested track and move to front of queue
    let track = backend.state.player.queuestate.queue.remove(index);
    backend.state.player.queuestate.queue.insert(0, track);

    // play the track
    playsong(backend)?;
    Ok(())
}

pub fn addplaylist(backend: &mut Backend, playlist: Playlist) {
    backend.state.playlists.push(playlist);
}

fn playsong(backend: &mut Backend) -> Result<()> {
    if backend.state.player.queuestate.queue.is_empty() {
        return Ok(());
    }

    let currenturl = &backend.state.player.queuestate.queue[0].url;

    // kill any existing mpv process
    if let Some(mut process) = backend.mpvprocess.take() {
        let _ = process.kill();
        // give the process time to fully terminate
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // make sure the socket file doesnt exist
    if std::path::Path::new(MPVSOCKET).exists() {
        let _ = std::fs::remove_file(MPVSOCKET);
    }

    // start new mpv process
    backend.mpvprocess = Some(
        Command::new("mpv")
            .arg(currenturl)
            .arg("--input-ipc-server=".to_string() + MPVSOCKET)
            .arg("--no-video")
            .arg("--no-terminal")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?,
    );

    // wait for socket to be created
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(3);

    while !std::path::Path::new(MPVSOCKET).exists() {
        if start.elapsed() > timeout {
            // if socket does not appear, warn
            eprintln!("warning: mpv socket not created after timeout");
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // more wait for socket to be ready after file exists
    if std::path::Path::new(MPVSOCKET).exists() {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    backend.state.player.isplaying = true;

    // reset current time
    backend.state.player.currenttime = 0;

    Ok(())
}

// --- navigation ---

pub fn nextcolumn(backend: &mut Backend) {
    let currentcolumn = &backend.selection.selectedcolumn;

    backend.selection.selectedcolumn = match currentcolumn {
        CurrentColumn::Playlists => {
            // only allow moving to tracks if there are playlists and a playlist is selected
            if !backend.state.playlists.is_empty()
                && backend.selection.playliststate.selected().is_some()
            {
                // make sure the tracks column has a selection if the playlist has tracks
                if let Some(playlistidx) = backend.selection.playliststate.selected() {
                    if let Some(playlist) = backend.state.playlists.get(playlistidx) {
                        if !playlist.tracks.is_empty()
                            && backend.selection.trackstate.selected().is_none()
                        {
                            backend.selection.trackstate.select(Some(0));
                        }
                    }
                }
                CurrentColumn::Tracks
            } else {
                CurrentColumn::Playlists
            }
        }
        CurrentColumn::Tracks => {
            // only allow moving to queue if there is something in the queue
            if !backend.state.player.queuestate.queue.is_empty() {
                if backend.selection.queuestate.selected().is_none() {
                    backend.selection.queuestate.select(Some(0));
                }
                CurrentColumn::Queue
            } else {
                CurrentColumn::Tracks
            }
        }
        CurrentColumn::Queue => CurrentColumn::Queue,
    };
}

pub fn prevcolumn(backend: &mut Backend) {
    let currentcolumn = &backend.selection.selectedcolumn;

    backend.selection.selectedcolumn = match currentcolumn {
        CurrentColumn::Playlists => CurrentColumn::Playlists,
        CurrentColumn::Tracks => CurrentColumn::Playlists,
        CurrentColumn::Queue => {
            // make sure track state is selected when moving back to tracks
            if backend.selection.trackstate.selected().is_none() {
                if let Some(playlistidx) = backend.selection.playliststate.selected() {
                    if let Some(playlist) = backend.state.playlists.get(playlistidx) {
                        if !playlist.tracks.is_empty() {
                            backend.selection.trackstate.select(Some(0));
                        }
                    }
                }
            }
            CurrentColumn::Tracks
        }
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

                // update track selection for the new playlist when changing playlists
                if next != current && backend.selection.trackstate.selected().is_some() {
                    if let Some(playlist) = backend.state.playlists.get(next) {
                        // reset track selection to first track in the new playlist
                        if !playlist.tracks.is_empty() {
                            backend.selection.trackstate.select(Some(0));
                        } else {
                            backend.selection.trackstate.select(None);
                        }
                    }
                }
            }
        }
        CurrentColumn::Tracks => {
            if let Some(playlistidx) = backend.selection.playliststate.selected() {
                if let Some(playlist) = backend.state.playlists.get(playlistidx) {
                    let trackslen = playlist.tracks.len();
                    if trackslen > 0 {
                        // if no track is selected, select the first one
                        if backend.selection.trackstate.selected().is_none() {
                            backend.selection.trackstate.select(Some(0));
                        } else {
                            let current = backend.selection.trackstate.selected().unwrap();
                            let next = if current + 1 >= trackslen {
                                trackslen - 1
                            } else {
                                current + 1
                            };
                            backend.selection.trackstate.select(Some(next));
                        }
                    }
                }
            }
        }
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
                let next = if current == 0 { 0 } else { current - 1 };
                backend.selection.playliststate.select(Some(next));

                // when changing playlists, update track selection for the new playlist
                if next != current && backend.selection.trackstate.selected().is_some() {
                    if let Some(playlist) = backend.state.playlists.get(next) {
                        // reset track selection to first track in the new playlist
                        if !playlist.tracks.is_empty() {
                            backend.selection.trackstate.select(Some(0));
                        } else {
                            backend.selection.trackstate.select(None);
                        }
                    }
                }
            }
        }
        CurrentColumn::Tracks => {
            if let Some(playlistidx) = backend.selection.playliststate.selected() {
                if let Some(playlist) = backend.state.playlists.get(playlistidx) {
                    if !playlist.tracks.is_empty() {
                        // if no track is selected, select the first one
                        if backend.selection.trackstate.selected().is_none() {
                            backend.selection.trackstate.select(Some(0));
                        } else {
                            let current = backend.selection.trackstate.selected().unwrap();
                            let next = if current == 0 { 0 } else { current - 1 };
                            backend.selection.trackstate.select(Some(next));
                        }
                    }
                }
            }
        }
        CurrentColumn::Queue => {
            if !backend.state.player.queuestate.queue.is_empty() {
                let current = backend.selection.queuestate.selected().unwrap_or(0);
                let next = if current == 0 { 0 } else { current - 1 };
                backend.selection.queuestate.select(Some(next));
            }
        }
    }
}
