use std::process::Command;
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

/// gets pid of mpv process
/// 
/// # returns
/// * 'pid' - pid of mpv process
pub fn getmpvpid() -> Result<String> {
    let output = Command::new("pidof")
        .arg("mpv")
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("failed to get mpv pid"));
    }

    let pid = String::from_utf8(output.stdout)?;

    Ok(pid)
}

/// unpauses mpv process
/// 
/// # arguments
/// * 'mpvpid' - pid of mpv process
/// 
/// # returns
/// * nothing
pub fn unpause(mpvpid: String) -> Result<()> {
    Command::new("kill")
        .arg("-s")
        .arg("CONT")
        .arg(mpvpid);
    Ok(())
}

/// pauses mpv process
/// 
/// # arguments
/// * 'mpvpid' - pid of mpv process
/// 
/// # returns
/// * nothing
pub fn pause(mpvpid: String) -> Result<()> {
    Command::new("kill")
        .arg("-s")
        .arg("STOP")
        .arg(mpvpid);
    Ok(())
}