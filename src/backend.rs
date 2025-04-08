use std::process::{Command, Stdio};
use anyhow::Result;

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