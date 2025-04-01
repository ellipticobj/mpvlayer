use std::process::Command;
use anyhow::Result;


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

pub fn getmetadata(url: &str) -> (String, String, String) {
    let duration = getduration(url).unwrap();
    let title = gettitle(url).unwrap();
    let artist = getartist(url).unwrap();

    (duration, title, artist)
}

pub fn playlisturlfromid(id: String) -> String {
    String::from(format!("https://www.youtube.com/playlist?list={}", id))
}

pub fn videourlfromid(id: String) -> String {
    String::from(format!("https://www.youtube.com/watch?v={}", id))
}