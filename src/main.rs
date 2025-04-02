use std::{
    io, time::Duration
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
    Track,
    Queue,
    RepeatType
};

impl App {
    fn ontick(&mut self) -> Result<()> {
        if self.playing {
            if self.mpv.is_none() {
                self.mpv = Some(std::process::Command::new("mpv").arg("--no-terminal").spawn()?);
            }
        }

        Ok(())
    }

    fn onkey(&mut self, key: KeyCode) {
        match key {
            // --- song control ---
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char(' ') => self.playing = !self.playing,
            KeyCode::Char('l') => {
                if !self.queue.queue.is_empty() {
                    if self.currentqueueidx < self.queue.queue.len() as u32 {
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
                        self.currentqueueidx = self.queue.queue.len() as u32 - 1;
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

            // --- navigation ---
            KeyCode::Up => {
                if self.currentlyselectedplaylist {
                    if !self.playlists.is_empty() {
                        if self.currentlyselectedplaylistidx > 0 {
                            self.currentlyselectedplaylistidx -= 1;
                        } else {
                            self.currentlyselectedplaylistidx = self.playlists.len() as u32 - 1;
                        }
                    }
                } else {
                    if !self.playlists.is_empty() && !self.playlists[self.currentlyselectedplaylistidx as usize].tracks.is_empty() {
                        if self.currentlyselectedtrackidx > 0 {
                            self.currentlyselectedtrackidx -= 1;
                        } else {
                            self.currentlyselectedtrackidx = self.playlists[self.currentlyselectedplaylistidx as usize].tracks.len() as u32 - 1;
                        }
                    }
                }
            },
            KeyCode::Down => {
                if self.currentlyselectedplaylist {
                    if !self.playlists.is_empty() {
                        if self.currentlyselectedplaylistidx < self.playlists.len() as u32 - 1 {
                            self.currentlyselectedplaylistidx += 1;
                        } else {
                            self.currentlyselectedplaylistidx = 0;
                        }
                    }
                } else {
                    if !self.playlists.is_empty() && !self.playlists[self.currentlyselectedplaylistidx as usize].tracks.is_empty() {
                        if self.currentlyselectedtrackidx < self.playlists[self.currentlyselectedplaylistidx as usize].tracks.len() as u32 - 1 {
                            self.currentlyselectedtrackidx += 1;
                        } else {
                            self.currentlyselectedtrackidx = 0;
                        }
                    }
                }
            },
            KeyCode::Left => {
                self.currentlyselectedplaylist = true;
            },
            KeyCode::Right => {
                self.currentlyselectedplaylist = false;
            },
            
            _ => {}
        }

    }
}

fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    terminal.draw(|frame| {
        constructors::rendermainview(app, frame, constructors::construct(frame.area()))
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
        duration: String::from("4:15"),
        url: String::from("https://www.youtube.com/watch?v=3FPwcaflCS8")
    };

    let dumdeedum = Track {
        title: String::from("dum dee dum"),
        artist: String::from("keys n' krates"),
        duration: String::from("3:03"),
        url: String::from("https://www.youtube.com/watch?v=eDshx6Rg9Hs")
    };

    let sigmaplaylist = Playlist {
        name: String::from("sigma"),
        tracks: vec![surfacebyaerochord, dumdeedum]
    };

    // --- initialize app ---
    let mut app = App {
        running: true,
        playing: false,
        playlists: vec![sigmaplaylist],
        queue: Queue { queue: Vec::new() },
        currentqueueidx: 0,
        currentplaylistidx: 0,
        shuffle: false,
        repeat: RepeatType::None,
        mpv: None, // dont init mpv yet, only start when user starts playing music
        currentlyselectedplaylist: true,
        currentlyselectedplaylistidx: 0,
        currentlyselectedtrackidx: 0
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