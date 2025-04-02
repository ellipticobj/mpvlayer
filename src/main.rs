use std::{
    io, process::Child, 
    time::Duration
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Terminal,
};
// use tui_framework_experiment::{Button, ButtonState, ButtonTheme};
use anyhow::Result;

mod ytdlp;
mod constructors;

struct Track {
    title: String,
    artist: String,
    duration: String,
    url: String
}

struct Playlist {
    name: String,
    tracks: Vec<Track>
}

struct Queue {
    tracks: Vec<Track>
}

struct App {
    running: bool,
    playing: bool,
    playlists: Vec<Playlist>,
    queue: Queue,
    currentqueidx: u16,
    currentplaylist: Playlist,
    shuffle: bool,
    repeat: bool,
    mpv: Option<Child>
}

impl App {
    fn ontick(&mut self) -> Result<()> {
        if self.playing {
            if self.mpv.is_none() {
                self.mpv = Some(std::process::Command::new("mpv").spawn()?);
            }
        }

        Ok(())
    }

    fn onkey(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char(' ') => self.playing = !self.playing,
            _ => {}
        }

    }
}

fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, _app: &mut App) -> Result<()> {
    terminal.draw(|frame| {
        constructors::drawmainview(frame, constructors::construct(frame.area()))
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

    // --- initialize app ---
    let mut app = App {
        running: true,
        playing: false,
        playlists: Vec::new(),
        queue: Queue { tracks: Vec::new() },
        currentqueidx: 0,
        currentplaylist: Playlist { name: String::new(), tracks: Vec::new() },
        shuffle: false,
        repeat: false,
        mpv: None // dont init mpv yet, only start when user starts playing music
    };
    
    // --- main loop ---
    while app.running {
        draw(&mut terminal, &mut app)?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.onkey(key.code);
                }
            }
        }

        app.ontick();
    }
    
    // -- cleanup ---
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}