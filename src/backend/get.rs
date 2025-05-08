// get.rs
// backend read api

use super::Backend;
use crate::models::{AppState, CurrentColumn, PlayerState, Playlist, RepeatMode, Track, MPVSOCKET};
use anyhow::Result;
use serde_json;
use std::process::{Command, Stdio};

// --- state access functions (for frontend to read) ---
pub fn state(backend: &Backend) -> &AppState {
    &backend.state
}

pub fn playingstate(backend: &Backend) -> &bool {
    &backend.state.player.isplaying
}

pub fn playerstate(backend: &Backend) -> &PlayerState {
    &backend.state.player
}

pub fn playlists(backend: &Backend) -> &Vec<Playlist> {
    &backend.state.playlists
}

pub fn repeatstate(backend: &Backend) -> &RepeatMode {
    &backend.state.player.repeatstate.repeatmode
}

pub fn shufflestate(backend: &Backend) -> &bool {
    &backend.state.player.shufflestate.shuffle
}

pub fn elapsedduration(backend: &Backend) -> Result<u32> {
    // if nothings playing or the socket doesn't exist
    if !*playingstate(backend) || !std::path::Path::new(MPVSOCKET).exists() {
        return Ok(0);
    }

    // timeout
    let timeout = std::time::Duration::from_secs(2);

    // unique request ID for this request
    let requestid = rand::random::<u32>();

    let echoout = Command::new("echo")
        .arg(format!(
            r#"{{"command":["get_property","time-pos"], "request_id": {}}}"#,
            requestid
        ))
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(echoout) = echoout.stdout {
        // send the command to mpv via socket and capture the output
        let socatout = Command::new("socat")
            .arg("-T")
            .arg(timeout.as_secs().to_string())
            .arg("-")
            .arg(MPVSOCKET)
            .stdin(Stdio::from(echoout))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        // if socat fails, just return 0 instead of an error
        if !socatout.status.success() {
            return Ok(0);
        }

        // parse the JSON response
        let response = String::from_utf8_lossy(&socatout.stdout);

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
            // extract the position value
            if let Some(data) = json.get("data") {
                if data.is_null() {
                    return Ok(0);
                }

                if let Some(position) = data.as_f64() {
                    return Ok(position.floor() as u32); // convert to u32 (seconds)
                }
            }
        }

        // parsing errors
        Ok(0)
    } else {
        // cant get stdout from echo
        Ok(0)
    }
}

pub fn totalduration(backend: &Backend) -> Result<u32> {
    // if nothing's playing or the socket doesn't exist, return 0 without error
    if !*playingstate(backend) || !std::path::Path::new(MPVSOCKET).exists() {
        return Ok(0);
    }

    // add timeout for socket operations
    let timeout = std::time::Duration::from_secs(2);
    let requestid = rand::random::<u32>();

    let echoout = Command::new("echo")
        .arg(format!(
            r#"{{"command":["get_property","duration"], "request_id": {}}}"#,
            requestid
        ))
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(echoout) = echoout.stdout {
        // send the command to mpv via socket and capture the output
        let socatout = Command::new("socat")
            .arg("-T")
            .arg(timeout.as_secs().to_string())
            .arg("-")
            .arg(MPVSOCKET)
            .stdin(Stdio::from(echoout))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        // if socat fails, just return 0 instead of an error
        if !socatout.status.success() {
            return Ok(0);
        }

        // parse the JSON response
        let response = String::from_utf8_lossy(&socatout.stdout);

        // parse the response JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
            // extract the position value
            if let Some(data) = json.get("data") {
                if data.is_null() {
                    return Ok(0);
                }

                if let Some(position) = data.as_f64() {
                    return Ok(position.floor() as u32); // convert to u32 (seconds)
                }
            }
        }

        // parsing errors
        Ok(0)
    } else {
        // cant get stdout from echo
        Ok(0)
    }
}

pub fn currentsong(backend: &Backend) -> Option<&Track> {
    backend.state.player.queuestate.queue.get(0)
}

pub fn queue(backend: &Backend) -> &Vec<Track> {
    &backend.state.player.queuestate.queue
}

pub fn playlisttracks(backend: &Backend, trackidx: usize) -> Option<Vec<Track>> {
    if trackidx < backend.state.playlists.len() {
        Some(backend.state.playlists[trackidx].tracks.clone())
    } else {
        None
    }
}

pub fn selectedcolumn(backend: &Backend) -> &CurrentColumn {
    &backend.selection.selectedcolumn
}

pub fn playlistsempty(backend: &Backend) -> bool {
    backend.state.playlists.is_empty()
}

pub fn queueempty(backend: &Backend) -> bool {
    backend.state.player.queuestate.queue.is_empty()
}

pub fn playliststate(backend: &Backend) -> &ratatui::widgets::ListState {
    &backend.selection.playliststate
}

pub fn trackstate(backend: &Backend) -> &ratatui::widgets::ListState {
    &backend.selection.trackstate
}

pub fn queuestate(backend: &Backend) -> &ratatui::widgets::ListState {
    &backend.selection.queuestate
}
