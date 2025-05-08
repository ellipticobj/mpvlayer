// main.rs
// entry point

mod backend;
mod frontend;
mod models;

use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::Result;
use backend::Backend;
use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use frontend::runfrontend;
use ratatui::{backend::CrosstermBackend, Terminal};

fn prerunchecks() -> Result<()> {
    // check for mpv and socat installations
    let mpvcheck = Command::new("mpv")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if let Err(_) = mpvcheck {
        return Err(anyhow::anyhow!("mpv is not installed or not in PATH"));
    }

    let socatcheck = Command::new("socat")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if let Err(_) = socatcheck {
        return Err(anyhow::anyhow!("socat is not installed or not in PATH"));
    }

    // clean up any existing socket file
    if std::path::Path::new(models::MPVSOCKET).exists() {
        let _ = std::fs::remove_file(models::MPVSOCKET);
    }

    Ok(())
}

fn main() -> Result<()> {
    // run pre-startup checks
    if let Err(e) = prerunchecks() {
        // if prerunchecks fail, start a minimal UI just to show the error
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backendtui = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backendtui)?;

        // create a minimal app instance just for the error popup
        let mut app = frontend::App::new(models::VERSION.to_string());

        // show a critical error popup
        frontend::newpopup(
            &mut app,
            " critical error ".to_string(),
            vec![
                " dependency check failed: ".to_string(),
                format!(" {} ", e.to_string()),
                "".to_string(),
                " press 'q' to quit.".to_string(),
            ],
            true,
        );

        // force the popup to be displayed
        terminal.draw(|frame| {
            frontend::renderpopup(&app, frame);
        })?;

        // wait for the user to press 'q'
        loop {
            if event::poll(Duration::from_millis(100))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == event::KeyEventKind::Press
                        && (key.code == event::KeyCode::Char('q')
                            || key.code == event::KeyCode::Esc)
                    {
                        break;
                    }
                }
            }
        }

        // clean up and exit
        execute!(io::stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        return Ok(());
    }
    // initialize backend (without starting mpv)
    let mut backend = Backend::new();

    // add sample playlists but don't start playback yet
    let testtrack = models::Track {
        title: "losing interest".to_string(),
        artist: "adore".to_string(),
        url: "https://www.youtube.com/watch?v=HtR4PkPJiBk".to_string(),
    };
    let testtrack1 = models::Track {
        title: "losing interest but again".to_string(),
        artist: "adore".to_string(),
        url: "https://www.youtube.com/watch?v=HtR4PkPJiBk".to_string(),
    };

    let playlist1 = models::Playlist {
        name: "sigma 1".to_string(),
        tracks: vec![testtrack.clone(), testtrack1.clone()],
    };

    let playlist2 = models::Playlist {
        name: "sigma 2".to_string(),
        tracks: vec![
            models::Track {
                title: "1".to_string(),
                artist: "adore".to_string(),
                url: "https://www.youtube.com/watch?v=HtR4PkPJiBk".to_string(),
            },
            models::Track {
                title: "2".to_string(),
                artist: "adore".to_string(),
                url: "https://www.youtube.com/watch?v=HtR4PkPJiBk".to_string(),
            },
        ],
    };

    backend::set::addplaylist(&mut backend, playlist1);
    backend::set::addplaylist(&mut backend, playlist2);

    // start frontend, passing the backend for interaction
    runfrontend(backend)?;

    Ok(())
}
