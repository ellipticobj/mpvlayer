use std::{
    io, time::Duration
};
use constructors::{construct, rendermainview};
use consts::PopupState;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend, widgets::ListState, Terminal
};
use anyhow::Result;
use fs4::fs_std::FileExt;
use crate::consts::{
    App, Playlist, Track, RepeatType, CurrentColumn, LOCKPATH
};

mod app;
mod backend;
mod constructors;
mod consts;


fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    terminal.draw(|frame| {
        rendermainview(app, frame, construct(frame.area(), &app.playing))
    })?;

    Ok(())
}

fn main() -> Result<()> {
    // --- setup terminal ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?; 
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // fake playlists for now
    let surfacebyaerochord = Track { 
        title: String::from("surface"),
        artist: String::from("aerochord"),
        duration: 255,
        url: String::from("https://www.youtube.com/watch?v=3FPwcaflCS8")
    };

    let dumdeedum = Track {
        title: String::from("dum dee dum"),
        artist: String::from("keys n' krates"),
        duration: 183,
        url: String::from("https://www.youtube.com/watch?v=eDshx6Rg9Hs")
    };

    let traproyalty = Track {
        title: String::from("trap royalty"),
        artist: String::from("very cool tutorials"),
        duration: 73,
        url: String::from("https://www.youtube.com/watch?v=bzQdrvKAwR8")
    };

    let goodbye = Track {
        title: String::from("goodbye"),
        artist: String::from("irokz"),
        duration: 240,
        url: String::from("https://www.youtube.com/watch?v=jJxJ8O_fMgg")
    };

    let glockinmyrawri = Track {
        title: String::from("glock in my rawri"),
        artist: String::from("randy!"),
        duration: 136,
        url: String::from("https://www.youtube.com/watch?v=lWiRuvoOdGc")
    };

    let tspmo = Track {
        title: String::from("tspmo"),
        artist: String::from("tyla da creata"),
        duration: 180,
        url: String::from("https://www.youtube.com/watch?v=oLbrmJLlvgM")
    };

    let sigmaplaylist = Playlist {
        name: String::from("sigma"),
        tracks: vec![surfacebyaerochord.clone(), dumdeedum.clone(), tspmo.clone()]
    };

    let sigmaplaylistcopy = Playlist {
        name: String::from("sigma copy"),
        tracks: vec![traproyalty.clone(), goodbye.clone(), glockinmyrawri.clone(), surfacebyaerochord.clone(), dumdeedum.clone(), tspmo.clone()]
    };

    // --- initialize app ---
    let mut app = App {
        running: true,
        playing: false,
        version: String::from("0.0.1"),
        repeatedinstance: false,
        playlists: vec![sigmaplaylist.clone(), sigmaplaylistcopy.clone()],
        queue: Vec::new(),
        queuebeforeshuffle: None,
        queuebeforerepeat: None,
        currentqueueidx: 0,
        currentplaylistidx: 0,
        currentdurationsecs: 0,
        shuffle: false,
        repeat: RepeatType::None,
        mpv: None,
        currentcolumn: CurrentColumn::Playlists,
        playliststate: ListState::default(),
        tracksstate: ListState::default(),
        queuestate: ListState::default(),
        lockfile: None,
        popup: PopupState {
            onscreen: false,
            dangerous: false,
            title: String::from(""),
            message: Vec::new()
        }
    };

    app::firstrun(&mut app)?;
    let mut counter: u8 = 0;

    // --- draw announcement popups, etc etc ---
    terminal.draw(|frame| {
        // constructors::showpopup(&mut app, frame, String::from("hello"), vec![String::from("world")], false);
    })?;

    // --- main loop ---
    while app.running {
        draw(&mut terminal, &mut app)?;

        // --- event Handling ---
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let keycode = key.code;
                    if app.repeatedinstance {
                        if key.code == KeyCode::Enter {
                            app.running = false;
                        }
                    } else {
                        app::onkey(&mut app, keycode)?;
                    }
                }
            }
        }

        if !app.repeatedinstance {
            app::ontick(&mut app, &counter)?;
            if counter >= 3 {
                counter = 0;
            } else {
                counter += 1;
            }
        } else {
            // Optional: Add a small sleep for repeated instance to prevent busy-waiting
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    // --- manual cleanup ---
    // unlock and remove lock file (only if this instance held the lock)
    if let Some(file) = app.lockfile.take() {
        let _ = file.unlock();
        let _ = std::fs::remove_file(LOCKPATH);
    }

    // kill mpv process 
    if let Some(mut child) = app.mpv.take() {
        let _ = child.kill();
        let _ = child.wait(); // wait for the process to actually exit
    }
    
    // restore terminal
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
