use anyhow::Result;
use crossterm::event::KeyCode;

use crate::consts::{App, CurrentColumn, LOCKPATH};
use crate::backend;
use std::fs::OpenOptions;
use std::io::Write;
use std::process;
use fs4::fs_std::FileExt;

pub fn getnextidx(currentopt: Option<usize>, listlen: usize) -> usize {
    if listlen == 0 {
        return 0;
    }
    match currentopt {
        Some(current) => {
            if current >= listlen - 1 {
                0 // wrap around to top
            } else {
                current + 1
            }
        }
        None => 0, // if nothing selected, select the first item
    }
}

pub fn getprevidx(currentopt: Option<usize>, listlen: usize) -> usize {
    if listlen == 0 {
        return 0;
    }
    match currentopt {
        Some(current) => {
            if current == 0 {
                listlen - 1 // wrap around to bottom
            } else {
                current - 1
            }
        }
        None => listlen - 1, // if nothing selected, select the last item
    }
}

pub fn firstrun(app: &mut App) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(LOCKPATH)?;

    match file.try_lock_exclusive() {
        Ok(_) => {
            file.set_len(0)?;
            writeln!(&file, "{}", process::id())?;
            app.lockfile = Some(file);
            println!("Instance {}: Lock acquired successfully.", std::process::id());

            app.repeatedinstance = false;

            // --- initial app state setup ---
            if !app.playlists.is_empty() {
                app.playliststate.select(Some(0));
                if !app.playlists[0].tracks.is_empty() {
                    app.tracksstate.select(Some(0));
                } else {
                    app.tracksstate.select(None);
                }
            } else {
                app.playliststate.select(None);
                app.tracksstate.select(None);
            }
            app.queuestate.select(None);

            Ok(())
        }
        Err(_) => {
            println!("Instance {}: Failed to acquire lock (already held?).", std::process::id()); // Debug
            app.repeatedinstance = true;
            app.lockfile = None;

            Ok(())
        }
    }
}

pub fn ontick(app: &mut App, counter: &u8) -> Result<()> {
    if let Some(child) = &mut app.mpv {
        if let Ok(Some(_)) = child.try_wait() {
            // mpv died
            if app.playing {
                backend::playcurrenttrack(app)?;
            }
        }
    }

    if counter == &3 { // every second
        if app.playing {
            // get current position from MPV instead of incrementing our own counter
            if let Ok(mpvduration) = backend::getplaybackpos(crate::consts::MPVSOCKET) {
                app.currentdurationsecs = mpvduration;
                
                // check if we've reached the end of the track
                let queueidx = app.currentqueueidx as usize;
                let queuevec = &app.queue;
                if queuevec.len() > queueidx {
                    let trackduration = queuevec[queueidx].duration;
                    // if we're near the end of the track, play the next one
                    // use a small buffer (1 second) to ensure we change tracks before the end
                    if mpvduration >= trackduration.saturating_sub(1) {
                        app.currentdurationsecs = 0;
                        backend::playnexttrack(app)?;
                    }
                }
            } else {
                app.currentdurationsecs += 1;
                
                // also check for end of track in fallback mode
                let queueidx = app.currentqueueidx as usize;
                let queuevec = &app.queue;
                if queuevec.len() > queueidx {
                    if app.currentdurationsecs > queuevec[queueidx].duration {
                        app.currentdurationsecs = 0;
                        backend::playnexttrack(app)?;
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn onkey(app: &mut App, key: KeyCode) -> Result<()> {
    // if a popup is on screen
    if app.popup.onscreen {
        if key == KeyCode::Enter {
            app.popup.onscreen = false;
            return Ok(());
        }
        return Ok(());
    }

    // if this is a repeated instance, only allow enter to close
    if app.repeatedinstance {
        if key == KeyCode::Enter {
            app.running = false;
            return Ok(());
        }
        return Ok(());
    }

    match key {
        // --- controls ---
        KeyCode::Char('q') => app.running = false,
        KeyCode::Char(' ') => backend::togglepause(app)?,
        KeyCode::Char('>') => backend::playnexttrack(app)?,
        KeyCode::Char('<') => backend::playprevtrack(app)?,
        KeyCode::Char('s') => backend::toggleshuffle(app)?,
        KeyCode::Char('r') => backend::cyclerepeat(app)?,
        
        // --- navigation ---
        KeyCode::Up | KeyCode::Down | KeyCode::Char('k') | KeyCode::Char('j') => {
            let isup = key == KeyCode::Up || key == KeyCode::Char('k');
            handleverticalnavigation(app, isup)?;
        },
        KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
            let isleft = key == KeyCode::Left || key == KeyCode::Char('h');
            handlehorizontalnavigation(app, isleft)?;
        },
        KeyCode::Enter => handleenter(app)?,
        _ => {}
    }
    Ok(())
}

// navigation functions
pub fn handleverticalnavigation(app: &mut App, isup: bool) -> Result<()> {
    match app.currentcolumn {
        CurrentColumn::Playlists => {
            let playlists = &app.playlists;
            if !playlists.is_empty() {
                let currentselection = app.playliststate.selected();
                let nextselection = if isup {
                    getprevidx(currentselection, playlists.len())
                } else {
                    getnextidx(currentselection, playlists.len())
                };
                app.playliststate.select(Some(nextselection));
            }
        }
        CurrentColumn::Tracks => {
            let playlistidx = app.playliststate.selected().unwrap_or(0) as usize;
            let playlists = &app.playlists;
            let trackstate = &app.tracksstate;
            // ensure playlist index is valid before accessing tracks
            if playlistidx < playlists.len() {
                let tracks = &playlists[playlistidx].tracks;
                if !tracks.is_empty() {
                    let currentselection = trackstate.selected();
                    let nextselection = if isup {
                        getprevidx(currentselection, tracks.len())
                    } else {
                        getnextidx(currentselection, tracks.len())
                    };
                    app.tracksstate.select(Some(nextselection));
                }
            }
        }
        CurrentColumn::Queue => {
            let tracks = &app.queue;
            let queuestate = &app.queuestate;
            if !tracks.is_empty() {
                let currentselection = queuestate.selected();
                let nextselection = if isup {
                    getprevidx(currentselection, tracks.len())
                } else {
                    getnextidx(currentselection, tracks.len())
                };
                app.queuestate.select(Some(nextselection));
            }
        }
    }
    Ok(())
}

pub fn handlehorizontalnavigation(app: &mut App, isleft: bool) -> Result<()> {
    // deselect tracks/queuestate when switching
    // match app.currentcolumn {
    //     CurrentColumn::Playlists => {},
    //     CurrentColumn::Tracks => app.tracksstate.select(None),
    //     CurrentColumn::Queue => app.queuestate.select(None),
    // }

    // determine and set the new column
    app.currentcolumn = match app.currentcolumn {
        CurrentColumn::Playlists => if isleft { CurrentColumn::Queue } else { CurrentColumn::Tracks },
        CurrentColumn::Tracks => if isleft { CurrentColumn::Playlists } else { CurrentColumn::Queue },
        CurrentColumn::Queue => if isleft { CurrentColumn::Tracks } else { CurrentColumn::Playlists },
    };

    // select an item in the newly focused column
    match app.currentcolumn {
        CurrentColumn::Playlists => {
            if !app.playlists.is_empty() {
                 // select last known or default to 0
                let idxtoselect = std::cmp::min(app.playliststate.selected().unwrap_or(0), app.playlists.len() - 1);
                app.playliststate.select(Some(idxtoselect));
            }
        }
        CurrentColumn::Tracks => {
            let playlistidx = app.playliststate.selected().unwrap_or(0);
             // check if playlist and its tracks are valid before selecting
            if playlistidx < app.playlists.len() && !app.playlists[playlistidx].tracks.is_empty() {
                // select last known or default to 0
                let idxtoselect = std::cmp::min(app.tracksstate.selected().unwrap_or(0), app.playlists[playlistidx].tracks.len() - 1);
                app.tracksstate.select(Some(idxtoselect));
            }
        }
        CurrentColumn::Queue => {
            if !app.queue.is_empty() {
                // select current playing index or default to 0
                let idxtoselect = std::cmp::min(app.currentqueueidx as usize, app.queue.len() - 1);
                app.queuestate.select(Some(idxtoselect));
            }
        }
    }
    Ok(())
}

pub fn handleenter(app: &mut App) -> Result<()> {
    match app.currentcolumn {
        CurrentColumn::Playlists => {
            // use selected index from state
            if let Some(selectedidx) = app.playliststate.selected() {
                if selectedidx < app.playlists.len() {
                    app.currentplaylistidx = selectedidx as u32; // update context
                    app.queue = app.playlists[selectedidx].tracks.clone();
                    app.currentqueueidx = 0; // start from beginning

                    let startidx = app.tracksstate.selected().unwrap_or(0);
                    app.currentqueueidx = startidx as u32;
                    app.queuestate.select(Some(startidx));

                    app.playing = true;
                    backend::playcurrenttrack(app)?;
                }
            }
        }
        CurrentColumn::Tracks => {
            if let Some(selectedtrackidx) = app.tracksstate.selected() {
                let playlistidx = app.playliststate.selected().unwrap_or(0) as usize;
                // check playlist index validity
                if playlistidx < app.playlists.len() {
                    let currentplaylist = &app.playlists[playlistidx].tracks;
                    // check track index validity
                    if selectedtrackidx < currentplaylist.len() {
                        app.currentplaylistidx = playlistidx as u32;
                        // create queue starting from selected track
                        app.queue = currentplaylist.clone();
                        // set current song to selected track
                        app.currentqueueidx = selectedtrackidx as u32;
                        app.queuestate.select(Some(selectedtrackidx as usize));
                        app.playing = true;
                        backend::playcurrenttrack(app)?;
                    }
                }
            }
        }
        CurrentColumn::Queue => {
            // use selected index from state
            if let Some(selected) = app.queuestate.selected() {
                // check queue index validity
                if selected < app.queue.len() {
                    app.currentqueueidx = selected as u32; // jump to selected track
                    app.playing = true;
                    backend::playcurrenttrack(app)?;
                }
            }
        }
    }
    backend::repeatqueue(app)?;
    backend::shufflequeue(app)?;
    Ok(())
}
