use std::{
    io, time::Duration
};
use constructors::{construct, rendermainview};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend, widgets::ListState, Terminal
};
use anyhow::Result;
use crate::consts::{
    App, Playlist, Track, RepeatType, CurrentColumn
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

impl App {
    fn onkey(&mut self, key: KeyCode) -> Result<()> {
        match key {
            // --- controls ---
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char(' ') => backend::togglepause(self)?,
            KeyCode::Char('>') => backend::playnexttrack(self)?,
            KeyCode::Char('<') => backend::playprevtrack(self)?,
            KeyCode::Char('s') => backend::toggleshuffle(self)?,
            KeyCode::Char('r') => backend::cyclerepeat(self)?,
            
            // --- navigation ---
            KeyCode::Up | KeyCode::Down | KeyCode::Char('k') | KeyCode::Char('j') => {
                let isup = key == KeyCode::Up || key == KeyCode::Char('k');
                app::handleverticalnavigation(self, isup)?;
            },
            KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                let isleft = key == KeyCode::Left || key == KeyCode::Char('h');
                app::handlehorizontalnavigation(self, isleft)?;
            },
            KeyCode::Enter => app::handleenter(self)?,
            _ => {}
        }
        Ok(())
    }
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
        playlists: vec![sigmaplaylist.clone(), sigmaplaylistcopy.clone()],
        queue: Vec::new(),
        queuebeforeshuffle: None,
        queuebeforerepeat: None,
        currentqueueidx: 0,
        currentplaylistidx: 0,
        currentdurationsecs: 0,
        shuffle: false,
        repeat: RepeatType::None,
        mpv: None, // don't init mpv yet, only start when user starts playing music
        currentcolumn: CurrentColumn::Playlists,
        playliststate: ListState::default(),
        tracksstate: ListState::default(),
        queuestate: ListState::default()
    };

    app::firstrun(&mut app)?;
    let mut counter: u8 = 0;

    // --- main loop ---
    while app.running {
        draw(&mut terminal, &mut app)?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.onkey(key.code)?;
                }
            }
        }
        app::ontick(&mut app, &counter)?;
        if counter >= 3 {
            counter = 0;
        } else {
            counter += 1;
        }
    }
    
    // -- cleanup ---
    backend::killallmpv();
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}