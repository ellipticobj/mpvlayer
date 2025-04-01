use std::{
    clone, error::Error, io, process::{
        Child, Command, Stdio
    }, 
    rc::Rc, time::{Duration, Instant}
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, LineGauge, List, ListItem, Paragraph},
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
    playlists: Vec<Playlist>,
    queue: Queue,
    currentqueidx: u16,
    currentplaylist: Playlist,
    shuffle: bool,
    repeat: bool,
    mpv: Child
}

impl App {
    fn ontick(&mut self) {
        
    }

    fn onkey(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.running = false,
            _ => {}
        }

    }
}

fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    terminal.draw(|frame| {
        let area = frame.area();
        let (upperlayout, lowerlayout, mainlayout) = constructors::mainlayout(area);
        constructors::upperview(area, upperlayout);
        constructors::lowerview(area, lowerlayout);
        constructors::mainview(area, mainlayout);
    })?;

    Ok(())
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mut app = App {
        running: true,
        playlists: Vec::new(),
        queue: Queue { tracks: Vec::new() },
        currentqueidx: 0,
        currentplaylist: Playlist { name: String::new(), tracks: Vec::new() },
        shuffle: false,
        repeat: false,
        mpv: Command::new("mpv").arg("--no-video").arg("--no-terminal").spawn()?
    };
    
    loop {
        draw(&mut terminal, &mut app)?;

    }

    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}