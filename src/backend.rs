use std::process::{Command, Stdio};
use anyhow::Result;
use rand::seq::SliceRandom;
use serde_json::Value;
use crate::consts::{App, RepeatType, Track, MAXQUEUELENGTH, MPVSOCKET};

/// gets duration of video using yt-dlp
/// 
/// # arguments
/// * 'url' - url of the video
/// 
/// # returns
/// * 'duration' - duration of the video in M:SS format
fn getduration(url: &str) -> Result<String> {
    let output = Command::new("yt-dlp")
        .arg("--get-duration")
        .arg(url)
        .arg("--no-warnings")
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("failed to get vids"));
    }

    let duration = String::from_utf8(output.stdout)?;

    Ok(duration)
}

/// gets title of video using yt-dlp
/// 
/// # arguments
/// * 'url' - url of the video
/// 
/// # returns
/// * 'title' - title of the video as a String
fn gettitle(url: &str) -> Result<String> {
    let output = Command::new("yt-dlp")
        .arg("--get-title")
        .arg(url)
        .arg("--no-warnings")
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("failed to get title"));
    }

    let title = String::from_utf8(output.stdout)?;

    Ok(title)
}

/// gets artist of video using yt-dlp
/// 
/// # arguments
/// * 'url' - url of the video
/// 
/// # returns
/// * 'artist' - artist of the video as a String
fn getartist(url: &str) -> Result<String> {
    let output = Command::new("yt-dlp")
        .arg("--print")
        .arg("'$(channel)s'")
        .arg(url)
        .arg("--no-warnings")
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("failed to get artist"));
    }

    let artist = String::from_utf8(output.stdout)?;

    Ok(artist)
}

/// gets list of video ids from playlist using yt-dlp
/// 
/// # arguments
/// * 'url' - url of the playlist
/// 
/// # returns
/// * 'ids' - list of video ids as a Vec<String>
pub fn getvidsfromplaylist(url: &str) -> Result<Vec<String>> {
    let output = Command::new("yt-dlp")
        .arg("--get-id")
        .arg(url)
        .arg("--no-warnings")
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("failed to get vids"));
    }

    let idstr = String::from_utf8(output.stdout)?;
    let ids = idstr.split("\n").map(|s| videourlfromid(s.to_string())).collect();

    Ok(ids)
}

/// gets metadata of video using yt-dlp
/// 
/// # arguments
/// * 'url' - url of the video
/// 
/// # returns
/// * 'duration' - duration of the video in M:SS format
/// * 'title' - title of the video as a String
/// * 'artist' - artist of the video as a String
pub fn getmetadata(url: &str) -> Result<(String, String, String), anyhow::Error> {
    let duration = getduration(url)?;
    let title = gettitle(url)?;
    let artist = getartist(url)?;

    Ok((duration, title, artist))
}

/// gets playlist url from id
/// 
/// # arguments
/// * 'id' - id of the playlist
/// 
/// # returns
/// * 'url' - url of the playlist as a String
pub fn playlisturlfromid(id: String) -> String {
    String::from(format!("https://www.youtube.com/playlist?list={}", id))
}

/// gets video url from id
/// 
/// # arguments
/// * 'id' - id of the video
/// 
/// # returns
/// * 'url' - url of the video as a String
pub fn videourlfromid(id: String) -> String {
    String::from(format!("https://www.youtube.com/watch?v={}", id))
}

/// pauses playback
/// 
/// # arguments 
/// * none
/// 
/// # returns
/// * none
pub fn pause(mpvsocket: &str) -> Result<()> {
    let echo_output = Command::new("echo")
        .arg(r#"{"command": ["cycle", "pause"]}"#)
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(echo_stdout) = echo_output.stdout {
        let socat_output = Command::new("socat")
            .arg("-")
            .arg(mpvsocket)
            .stdin(Stdio::from(echo_stdout)) 
            .stdout(Stdio::null()) 
            .stderr(Stdio::piped()) 
            .output()?;

        if !socat_output.status.success() {
            let stderr = String::from_utf8_lossy(&socat_output.stderr);
            eprintln!("failed to send pause command to mpv: {}", stderr);
        }
    } else {
        eprintln!("failed to get stdout from echo command");
    }

    Ok(())
}

/// Gets the current playback position from mpv in seconds
/// 
/// # arguments
/// * 'mpvsocket' - path to the mpv socket
/// 
/// # returns
/// * 'position' - current playback position in seconds as u32
pub fn getplaybackpos(mpvsocket: &str) -> Result<u32> {
    // unique request ID for this request
    let requestid = rand::random::<u32>();
    
    let echoout = Command::new("echo")
        .arg(format!(r#"{{"command":["get_property","time-pos"], "request_id": {}}}"#, requestid))
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(echo_stdout) = echoout.stdout {
        // send the command to mpv via socket and capture the output
        let socatout = Command::new("socat")
            .arg("-")
            .arg(mpvsocket)
            .stdin(Stdio::from(echo_stdout))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !socatout.status.success() {
            let stderr = String::from_utf8_lossy(&socatout.stderr);
            eprintln!("failed to get playback position from mpv: {}", stderr);
            return Ok(0);
        }

        // parse the JSON response
        let response = String::from_utf8_lossy(&socatout.stdout);
        
        // parse the response JSON
        if let Ok(json) = serde_json::from_str::<Value>(&response) {
            // Extract the position value
            if let Some(data) = json.get("data") {
                if let Some(position) = data.as_f64() {
                    return Ok(position.floor() as u32); // convert to u32 (seconds)
                }
            }
        }
        
        // return 0 if we couldn't parse the position
        return Ok(0);
    } else {
        eprintln!("failed to get stdout from echo command");
        return Ok(0); // return 0 on error
    }
}

/// kills all currently running mpv processes using kill
/// 
/// # arguments
/// * none
/// 
/// # returns
/// * none
pub fn killallmpv() {
    let output = Command::new("pidof")
        .arg("mpv")
        .output()
        .unwrap();

    if !&output.status.success() {
        return;
    }

    for pid in output.stdout.split(|&x| x == b' ').into_iter().map(|x| std::str::from_utf8(x).unwrap()).into_iter() {
        let output = Command::new("kill")
            .arg(pid)
            .output();

        if !output.unwrap().status.success() {
            eprintln!("failed to kill mpv with pid {}", pid);
        }
    }
}

/// plays the current track using mpv
/// 
/// spawns an mpv child process to play the track
/// 
/// # arguments
/// * 'app' - mutable reference to the app state
/// 
/// # returns
/// * none
pub fn playcurrenttrack(app: &mut App) -> Result<()> {
    // --- safety checks ---
    let trackidx = app.currentqueueidx as usize;

    if app.queue.is_empty() || trackidx >= app.queue.len() {
        // if there is nothing to play
        app.playing = false;
        app.currentdurationsecs = 0;

        // kill any existing child processes
        if let Some(mut child) = app.mpv.take() {
            let _ = child.kill().map_err(|e| eprintln!("failed to kill child: {}", e));
        }
        return Ok(());
    }

    // --- kill previous child mpv instance ---
    if let Some(mut child) = app.mpv.take() {
        match child.kill() {
            Ok(_) => { /* succesfully killed */ }
            Err(e) => eprintln!("failed to kill child: {}", e),
        }
        // child.wait()?; // if issues occur
    }

    // --- get url ---
    let trackurl = &app.queue[trackidx].url;
    // let tracktitle = &app.queue[track_idx].title;

    // --- reset progress timer ---
    app.currentdurationsecs = 0;

    // println!("attempting to play: '{}' from {}", tracktitle, trackurl); // debug print
    let childproc = std::process::Command::new("mpv")
        .arg("--no-video")
        .arg("--no-terminal")
        .arg(format!("--input-ipc-server={}", MPVSOCKET))
        .arg("--pause=no")
        .arg("--keep-open=yes")
        // .arg("--no-audio-display")? .arg("--vo=null")? // audio-only if needed 
        // .arg("--really-quiet") // quieter output 
        .arg(trackurl)
        .stdout(std::process::Stdio::null())  // discard stdout
        .stderr(std::process::Stdio::null())  // discard stderr
        .spawn() // start the process
        .map_err(|e| anyhow::anyhow!("failed to spawn mpv for url '{}': {}", trackurl, e))?;
    
    std::thread::sleep(std::time::Duration::from_millis(100));
    app.mpv = Some(childproc);
    Ok(())
}

/// plays the next track
/// 
/// # arguments
/// * 'app' - mutable reference to the app state
/// 
/// # returns
/// * none
pub fn playnexttrack(app: &mut App) -> Result<()> {
    if app.queue.is_empty() {
        return Ok(());
    }

    let nextidx = if app.currentqueueidx == app.queue.len() as u32 - 1 {
        0
    } else {
        app.currentqueueidx + 1
    };

    app.currentqueueidx = nextidx;
    app.queuestate.select(Some(nextidx as usize));
    app.playing = true;
    playcurrenttrack(app)?;
    Ok(())
}

/// plays the previous track
/// 
/// # arguments
/// * 'app' - mutable reference to the app state
/// 
/// # returns
/// * none
pub fn playprevtrack(app: &mut App) -> Result<()> {
    if app.queue.is_empty() {
        return Ok(());
    }
    let previdx = if app.currentqueueidx == 0 {
        app.queue.len() as u32 - 1   
    } else {
        app.currentqueueidx - 1
    };
    app.currentqueueidx = previdx;
    app.queuestate.select(Some(previdx as usize));
    app.playing = true;
    playcurrenttrack(app)?;

    Ok(())
}

/// pauses mpv if app.playing is true
/// 
/// # arguments
/// * 'app' - mutable reference to the app state
/// 
/// # returns
/// * none
pub fn togglepause(app: &mut App) -> Result<()> {
    if app.playing {
        pause(MPVSOCKET)?;
        app.playing = !app.playing;
    }
    Ok(())
}

/// toggles shuffle
/// 
/// # arguments
/// * 'app' - mutable reference to the app state
/// 
/// # returns
/// * none
pub fn toggleshuffle(app: &mut App) -> Result<()> {
    app.shuffle = !app.shuffle;
    if !app.queue.is_empty() {
        shufflequeue(app)?;
    }
    Ok(())
}

/// cycles the repeat type
/// 
/// # arguments
/// * 'app' - mutable reference to the app state
/// 
/// # returns
/// * none
pub fn cyclerepeat(app: &mut App) -> Result<()> {
    match app.repeat {
        RepeatType::None => app.repeat = RepeatType::All,
        RepeatType::All => app.repeat = RepeatType::One,
        RepeatType::One => app.repeat = RepeatType::None
    }
    repeatqueue(app)?;
    Ok(())
}

/// repeats the queue
/// 
/// # arguments
/// * 'app' - mutable reference to the app state
/// 
/// # returns
/// * none
pub fn repeatqueue(app: &mut App) -> Result<()> {
    if !app.queue.is_empty() {
        match app.repeat {
            RepeatType::All => {
                // if queue is not at max length, repeat current queue until it reaches MAXQUEUELENGTH
                if app.queue.len() < MAXQUEUELENGTH {
                    let originallen = app.queue.len();
                    while app.queue.len() < MAXQUEUELENGTH {
                        let spacetofill = MAXQUEUELENGTH - app.queue.len();
                        // chunks to add is the minimum of the remaining space and the original length
                        let chunkstoadd = std::cmp::min(originallen, spacetofill);
                        // append the extension to the queue
                        let mut extension: Vec<Track> = app.queue[0..chunkstoadd].to_vec();
                        app.queue.append(&mut extension);
                    }
                }
            },
            RepeatType::One => {
                // store current queue for later
                app.queuebeforerepeat = Some(app.queue.clone());
                // get current song
                let currentsong = app.queue[app.currentqueueidx as usize].clone();
                // repeat current song MAXQUEUELENGTH times
                app.queue = vec![currentsong; MAXQUEUELENGTH as usize];
            },
            RepeatType::None => {
                // for no repeat, we need to ensure the queue is in its original state
                if let Some(ref original) = app.queuebeforerepeat {
                    app.queue = original.clone();
                } else {
                    // if no original queue exists, use the current playlist
                    let playlistidx = app.currentplaylistidx as usize;
                    if playlistidx < app.playlists.len() {
                        app.queue = app.playlists[playlistidx].tracks.clone();
                    }
                }
            }
        }
    }

    Ok(())
}

/// shuffles the queue
/// 
/// # arguments
/// * 'app' - mutable reference to the app state
/// 
/// # returns
/// * none
pub fn shufflequeue(app: &mut App) -> Result<()> {
    if !app.queue.is_empty() {
        if app.shuffle {
            app.queuebeforeshuffle = Some(app.queue.clone());
            // Use the rng function from the rand crate
            let mut rng = rand::rng();
            app.queue.shuffle(&mut rng);
            app.currentqueueidx = 0;
            app.currentdurationsecs = 0;
            app.queuestate.select(Some(0));
        } else {
            if let Some(originalqueue) = app.queuebeforeshuffle.clone() {
                app.queue = originalqueue;
            } else {
                app.queue = app.playlists[app.currentplaylistidx as usize].clone().tracks;
            }
            app.queuebeforeshuffle = None;
            app.currentqueueidx = 0;
            app.currentdurationsecs = 0;
            app.queuestate.select(Some(0));
        }
    }

    Ok(())
}
