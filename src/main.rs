use std::{
    io, time::Duration
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend, widgets::ListState, Terminal
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
    fn new() -> App {
        let mut playlistsstate = ListState::default();
        playlistsstate.select(Some(0));

        let mut tracksstate = ListState::default();
        tracksstate.select(Some(0));

        App {
            running: true,
            playing: false,
            playlists: Vec::new(),
            queue: Queue { queue: Vec::new() },
            currentqueueidx: 0,
            currentplaylistidx: 0,
            shuffle: false,
            repeat: RepeatType::None,
            mpv: None,
            currentlyselectedplaylist: true,
            currentlyselectedplaylistidx: 0,
            currentlyselectedtrackidx: 0,
            playlistsstate,
            tracksstate
        }
    }

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
                        let i = match self.playlistsstate.selected() {
                            Some(i) => {
                                if i == 0 {
                                    self.playlists.len() - 1
                                } else {
                                    i
                                }
                            }
                            None => 0,
                        };
                        self.playlistsstate.select(Some(i));
                        self.currentlyselectedplaylistidx = i as u32;
                    }
                } else {
                    if !self.playlists.is_empty() && !self.playlists[self.currentlyselectedplaylistidx as usize].tracks.is_empty() {
                        let i = match self.tracksstate.selected() {
                            Some(i) => {
                                if i == 0 {
                                    self.playlists[self.currentlyselectedplaylistidx as usize].tracks.len() - 1
                                } else {
                                    i
                                }
                            }
                            None => 0,
                        };
                        self.tracksstate.select(Some(i));
                    }
                }
            },
            KeyCode::Down => {
                if self.currentlyselectedplaylist {
                    if !self.playlists.is_empty() {
                        let i = match self.playlistsstate.selected() {
                            Some(i) => {
                                if i >= self.playlists.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        self.playlistsstate.select(Some(i));
                        self.currentlyselectedplaylistidx = i as u32;
                    }
                } else {
                    if !self.playlists.is_empty() && !self.playlists[self.currentlyselectedplaylistidx as usize].tracks.is_empty() {
                        let i = match self.tracksstate.selected() {
                            Some(i) => {
                                if i >= self.playlists[self.currentlyselectedplaylistidx as usize].tracks.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        self.tracksstate.select(Some(i));
                    }
                }
            },
            KeyCode::Left => {
                self.currentlyselectedplaylist = true;
                self.tracksstate.select(None);
                self.playlistsstate.select(Some(self.currentlyselectedplaylistidx as usize));
            },
            KeyCode::Right => {
                self.currentlyselectedplaylist = false;
                self.playlistsstate.select(None);
                self.tracksstate.select(Some(self.currentlyselectedtrackidx as usize));
            },
            KeyCode::Enter => {
                if self.currentlyselectedplaylist {
                    self.currentplaylistidx = self.currentlyselectedplaylistidx;
                    self.queue.queue = self.playlists[self.currentlyselectedplaylistidx as usize].tracks.clone();
                } else {
                    self.queue.queue.push(self.playlists[self.currentlyselectedplaylistidx as usize].tracks[self.currentlyselectedtrackidx as usize].clone());
                }
                self.currentqueueidx = 0;
            }
            
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

    let traproyalty = Track {
        title: String::from("trap royalty"),
        artist: String::from("very cool tutorials"),
        duration: String::from("1:13"),
        url: String::from("https://www.youtube.com/watch?v=zouSbflXfOo")
    };

    let goodbye = Track {
        title: String::from("goodbye"),
        artist: String::from("irokz"),
        duration: String::from("4:00"),
        url: String::from("https://www.youtube.com/watch?v=jJxJ8O_fMgg")
    };

    let glockinmyrawri = Track {
        title: String::from("glock in my rawri"),
        artist: String::from("randy!"),
        duration: String::from("2:16"),
        url: String::from("https://www.youtube.com/watch?v=lWiRuvoOdGc")
    };

    let sigmaplaylist = Playlist {
        name: String::from("sigma"),
        tracks: vec![surfacebyaerochord.clone(), dumdeedum.clone()]
    };

    let sigmaplaylistcopy = Playlist {
        name: String::from("sigma copy"),
        tracks: vec![traproyalty.clone(), goodbye.clone(), glockinmyrawri.clone()]
    };

    // --- initialize app ---
    let mut app = App {
        running: true,
        playing: false,
        playlists: vec![sigmaplaylist.clone(), sigmaplaylistcopy.clone()],
        queue: Queue { queue: Vec::new() },
        currentqueueidx: 0,
        currentplaylistidx: 0,
        shuffle: false,
        repeat: RepeatType::None,
        mpv: None, // dont init mpv yet, only start when user starts playing music
        currentlyselectedplaylist: true,
        currentlyselectedplaylistidx: 0,
        currentlyselectedtrackidx: 0,
        playlistsstate: ListState::default(),
        tracksstate: ListState::default()
    };
    app.playlistsstate.select(Some(0));
    app.tracksstate.select(None);
    
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