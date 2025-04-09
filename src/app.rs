use anyhow::Result;

use crate::consts::{App, CurrentColumn};
use crate::backend;

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
    if !app.playlists.is_empty() {
        // if playlists exist
        app.playliststate.select(Some(0));
        
        if !app.playlists[0].tracks.is_empty() {
            // if playlist is not empty
            app.tracksstate.select(Some(0));
        } else {
            app.tracksstate.select(None);
        }
    } else {
        app.playliststate.select(None);
        app.tracksstate.select(None);
    }
    
    app.queuestate.select(None);
    backend::killallmpv();

    Ok(())
}

pub fn ontick(app: &mut App, counter: &u8) -> Result<()> {
    if counter == &3 { // every second
        if app.playing {app.currentdurationsecs += 1;} // add 1 second if playing
    } else {
        if !app.playing {
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
                    app.queuestate.select(Some(0));
                    app.playing = true;
                    backend::playcurrenttrack(app)?;
                }
            }
        }
        CurrentColumn::Tracks => {
            if let Some(selectedtrackidx) = app.tracksstate.selected() {
                let playlistidx = app.playliststate.selected().unwrap_or(0) as usize;
                let tracks = &app.playlists[playlistidx].tracks;
                // check playlist index validity
                if playlistidx < app.playlists.len() && selectedtrackidx < tracks.len() {
                    app.currentplaylistidx = playlistidx as u32; // update context
                    // set queue starting from selected track
                    app.queue = tracks[selectedtrackidx..].to_vec();
                    app.currentqueueidx = 0; // start from beginning of new queue
                    app.queuestate.select(Some(0));
                    app.playing = true;
                    backend::playcurrenttrack(app)?;
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
