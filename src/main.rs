use std::{
    io,
    time::Duration
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
// use tui_framework_experiment::{Button, ButtonState, ButtonTheme};
use anyhow::Result;

mod ytdlp;
mod constructors;
mod consts;

use consts::{
    App,
    Playlist,
    Queue,
    RepeatType
};

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
            KeyCode::Char('l') => {
                if !self.queue.queue.is_empty() {
                    if self.currentqueueidx < self.queue.queue.len() as u16 {
                        self.currentqueueidx += 1;
                    } else {
                        self.currentqueueidx = 0;
                    }
                }
            },
            KeyCode::Char('j') => {
                if !self.queue.queue.is_empty() {                
                    if self.currentqueueidx > 0 {
                        self.currentqueueidx -= 1;
                    } else {
                        self.currentqueueidx = self.queue.queue.len() as u16 - 1;
                    }
                }
            }
            KeyCode::Char('s') => self.shuffle = !self.shuffle,
            KeyCode::Char('o') => {
                if self.repeat != RepeatType::One {
                    self.repeat = RepeatType::One;
                } else {
                    self.repeat = RepeatType::None;
                }
            },
            KeyCode::Char('a') => {
                if self.repeat != RepeatType::All {
                    self.repeat = RepeatType::All;
                } else {
                    self.repeat = RepeatType::None;
                }
            },
            _ => {}
        }

    }
}

fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    terminal.draw(|frame| {
        constructors::drawmainview(app, frame, constructors::construct(frame.area()))
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
        queue: Queue { queue: Vec::new() },
        currentqueueidx: 0,
        currentplaylist: Playlist { name: String::new(), tracks: Vec::new() },
        shuffle: false,
        repeat: RepeatType::None,
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

        app.ontick()?;
    }
    
    // -- cleanup ---
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}