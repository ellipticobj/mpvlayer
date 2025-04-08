use std::{
    io::{self, prelude::*}, os::unix::net::UnixStream, process::{Command, Stdio}, time::Duration, usize
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

mod backend;
mod constructors;
mod consts;

use consts::{
    App,
    Playlist,
    Track,
    Queue,
    RepeatType,
    CurrentColumn
};

static MPVSOCKET: &str = "/tmp/mpvsocket";

impl App {
    fn getnextidx(currentopt: Option<usize>, listlen: usize) -> usize {
        if listlen == 0 {
            return 0;
        }
        match currentopt {
            Some(current) => {
                if current >= listlen - 1 {
                    0 // wrap around to top
                } else {
                    current + 1
                }
            }
            None => 0, // if nothing selected, select the first item
        }
    }

    fn getpreviousidx(currentopt: Option<usize>, listlen: usize) -> usize {
        if listlen == 0 {
            return 0;
        }
        match currentopt {
            Some(current) => {
                if current == 0 {
                    listlen - 1 // wrap around to bottom
                } else {
                    current - 1
                }
            }
            None => listlen - 1, // if nothing selected, select the last item
        }
    }

    fn firstrun(&mut self) -> Result<()> {
        self.playlistsstate.select(Some(0));
        self.tracksstate.select(None);
        self.queuestate.select(None);
        backend::killallmpv();

        Ok(())
    }

    fn ontick(&mut self, counter: &u8) -> Result<()> {
        if self.playing {
            if counter == &3 {
                self.currentdurationsecs += 1;
            }
        } else {
            let queueidx = self.currentqueueidx as usize;
            let queuevec = &self.queue.queue;
            if queuevec.len() > queueidx {
                if self.currentdurationsecs > queuevec[queueidx].duration {
                    self.currentdurationsecs = 0;
                    self.playnexttrack()?;
                }
            }
        }
        Ok(())
    }

    fn onkey(&mut self, key: KeyCode) -> Result<()> {
        match key {
            // --- controls ---
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char(' ') => self.pause()?,
            KeyCode::Char('l') => { // next track in queue
                
            }
            KeyCode::Char('j') => { // previous track in queue
                
            }
            KeyCode::Char('s') => self.shuffle = !self.shuffle, // toggle shuffle
            KeyCode::Char('r') => { // cycle repeat one/none/all
                match self.repeat {
                    RepeatType::None => {
                        self.repeat = RepeatType::All;
                    },
                    RepeatType::All => {
                        self.repeat = RepeatType::One;
                    },
                    RepeatType::One => {
                        self.repeat = RepeatType::None;
                    }
                }
            }

            // --- navigation ---
            KeyCode::Up | KeyCode::Down => {
                let isup = key == KeyCode::Up;
                match self.currentlyselectedcolumn {
                    CurrentColumn::Playlists => {
                        let playlists = &self.playlists;
                        if !playlists.is_empty() {
                            let currentselection = self.playlistsstate.selected();
                            let nextselection = if isup {
                                Self::getpreviousidx(currentselection, playlists.len())
                            } else {
                                Self::getnextidx(currentselection, playlists.len())
                            };
                            self.playlistsstate.select(Some(nextselection));
                            // keep currentlyselectedplaylistidx updated
                            self.currentlyselectedplaylistidx = nextselection as u32;
                        }
                    }
                    CurrentColumn::Tracks => {
                        let playlistidx = self.currentlyselectedplaylistidx as usize;
                        let playlists = &self.playlists;
                        let trackstate = &self.tracksstate;
                        // ensure playlist index is valid before accessing tracks
                        if playlistidx < playlists.len() {
                            let tracks = &playlists[playlistidx].tracks;
                            if !tracks.is_empty() {
                                let currentselection = trackstate.selected();
                                let nextselection = if isup {
                                    Self::getpreviousidx(currentselection, tracks.len())
                                } else {
                                    Self::getnextidx(currentselection, tracks.len())
                                };
                                self.tracksstate.select(Some(nextselection));
                                self.currentlyselectedtrackidx = nextselection as u32;
                            }
                        }
                    }
                    CurrentColumn::Queue => {
                        let tracks = &self.queue.queue;
                        let queuestate = &self.queuestate;
                        if !tracks.is_empty() {
                            let currentselection = queuestate.selected();
                            let nextselection = if isup {
                                Self::getpreviousidx(currentselection, tracks.len())
                            } else {
                                Self::getnextidx(currentselection, tracks.len())
                            };
                            self.queuestate.select(Some(nextselection));
                        }
                    }
                }
            }
            KeyCode::Left | KeyCode::Right => {
                let isleft = key == KeyCode::Left;

                // deselect current column's state
                match self.currentlyselectedcolumn {
                    CurrentColumn::Playlists => self.playlistsstate.select(None),
                    CurrentColumn::Tracks => self.tracksstate.select(None),
                    CurrentColumn::Queue => self.queuestate.select(None),
                }

                // determine and set the new column
                self.currentlyselectedcolumn = match self.currentlyselectedcolumn {
                    CurrentColumn::Playlists => if isleft { CurrentColumn::Queue } else { CurrentColumn::Tracks },
                    CurrentColumn::Tracks => if isleft { CurrentColumn::Playlists } else { CurrentColumn::Queue },
                    CurrentColumn::Queue => if isleft { CurrentColumn::Tracks } else { CurrentColumn::Playlists },
                };

                // select an item in the newly focused column
                match self.currentlyselectedcolumn {
                    CurrentColumn::Playlists => {
                        if !self.playlists.is_empty() {
                             // select last known or default to 0
                            let idxtoselect = std::cmp::min(self.currentlyselectedplaylistidx as usize, self.playlists.len() - 1);
                            self.playlistsstate.select(Some(idxtoselect));
                        }
                    }
                    CurrentColumn::Tracks => {
                        let playlistidx = self.currentlyselectedplaylistidx as usize;
                         // check if playlist and its tracks are valid before selecting
                        if playlistidx < self.playlists.len() && !self.playlists[playlistidx].tracks.is_empty() {
                            // select last known or default to 0
                            let idxtoselect = std::cmp::min(self.currentlyselectedtrackidx as usize, self.playlists[playlistidx].tracks.len() - 1);
                            self.tracksstate.select(Some(idxtoselect));
                        }
                    }
                    CurrentColumn::Queue => {
                        if !self.queue.queue.is_empty() {
                            // select current playing index or default to 0
                            let idxtoselect = std::cmp::min(self.currentqueueidx as usize, self.queue.queue.len() - 1);
                            self.queuestate.select(Some(idxtoselect));
                        }
                    }
                }
            }
            KeyCode::Enter => {
                match self.currentlyselectedcolumn {
                    CurrentColumn::Playlists => {
                        // use selected index from state
                        if let Some(selectedidx) = self.playlistsstate.selected() {
                            if selectedidx < self.playlists.len() {
                                self.currentplaylistidx = selectedidx as u32; // update context
                                self.queue.queue = self.playlists[selectedidx].tracks.clone();
                                self.currentqueueidx = 0; // start from beginning
                                self.queuestate.select(Some(0));
                                self.playing = true;
                                self.playcurrenttrack()?;
                            }
                        }
                    }
                    CurrentColumn::Tracks => {
                        if let Some(selectedtrackidx) = self.tracksstate.selected() {
                            let playlistidx = self.currentlyselectedplaylistidx as usize;
                            // check playlist index validity
                            if playlistidx < self.playlists.len() {
                                let tracks = &self.playlists[playlistidx].tracks;
                                // check track index validity
                                if selectedtrackidx < tracks.len() {
                                    self.currentplaylistidx = playlistidx as u32; // update context
                                    // set queue starting from selected track
                                    self.queue.queue = tracks[selectedtrackidx..].to_vec();
                                    self.currentqueueidx = 0; // start from beginning of new queue
                                    self.queuestate.select(Some(0));
                                    self.playing = true;
                                    self.playcurrenttrack()?;
                                }
                            }
                        }
                    }
                    CurrentColumn::Queue => {
                        // use selected index from state
                        if let Some(selectedidx) = self.queuestate.selected() {
                            // check queue index validity
                            if selectedidx < self.queue.queue.len() {
                                self.currentqueueidx = selectedidx as u32; // jump to selected track
                                self.playing = true;
                                self.playcurrenttrack()?;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn playcurrenttrack(&mut self) -> Result<()> {
        // --- safety checks ---
        let trackidx = self.currentqueueidx as usize;

        if self.queue.queue.is_empty() || trackidx >= self.queue.queue.len() {
            // if there is nothing to play
            self.playing = false;
            self.currentdurationsecs = 0;
    
            // kill any existing child processes
            if let Some(mut child) = self.mpv.take() {
                let _ = child.kill().map_err(|e| eprintln!("failed to kill child: {}", e));
            }
            return Ok(());
        }
    
        // --- kill previous child mpv instance ---
        if let Some(mut child) = self.mpv.take() {
            match child.kill() {
                Ok(_) => { /* succesfully killed */ }
                Err(e) => eprintln!("failed to kill child: {}", e),
            }
            // child.wait()?; // if issues occur
        }
    
        // --- get url ---
        let trackurl = &self.queue.queue[trackidx].url;
        let tracktitle = &self.queue.queue[trackidx].title;
    
        // --- reset progress timer ---
        self.currentdurationsecs = 0;
    
        println!("attempting to play: '{}' from {}", tracktitle, trackurl); // debug print
        let childproc = Command::new("mpv")
            .arg("--no-video")
            .arg("--no-terminal")
            .arg(format!("--input-ipc-server={}", MPVSOCKET))
            .arg("--pause=no")
            .arg("--keep-open=yes")
            // .arg("--no-audio-display")? .arg("--vo=null")? // audio-only if needed 
            // .arg("--really-quiet") // quieter output 
            .arg(trackurl)
            .stdout(Stdio::null())  // discard stdout
            .stderr(Stdio::null())  // discard stderr
            .spawn() // start the process
            .map_err(|e| anyhow::anyhow!("failed to spawn mpv for url '{}': {}", trackurl, e))?;
        
        std::thread::sleep
        (std::time::Duration::from_millis(100));
        self.mpv = Some(childproc);
        Ok(())
    }

    fn playnexttrack(&mut self) -> Result<()> {
        if self.queue.queue.is_empty() {
            return Ok(());
        }
        let nextidx = match self.repeat {
            RepeatType::None => {
                if self.currentqueueidx >= self.queue.queue.len() as u32 - 1 {
                    // no more songs to play
                    self.playing = false;
                    self.currentdurationsecs = 0;
                    return Ok(());
                } else {
                    self.currentqueueidx + 1
                }
            },
            RepeatType::One => self.currentqueueidx,
            RepeatType::All =>  {
                if self.currentqueueidx >= self.queue.queue.len() as u32 - 1 {
                    0
                } else {
                    self.currentqueueidx + 1
                }
            }
        };

        self.currentqueueidx = nextidx;
        self.queuestate.select(Some(nextidx as usize));
        self.playing = true;
        self.playcurrenttrack()?;
        Ok(())
    }

    fn playprevtrack(&mut self) -> Result<()> {
        if self.queue.queue.is_empty() {
            return Ok(());
        }
        let previdx = if self.currentqueueidx == 0 {
            self.queue.queue.len() as u32 - 1   
        } else {
            self.currentqueueidx - 1
        };
        self.currentqueueidx = previdx;
        self.queuestate.select(Some(previdx as usize));
        self.playing = true;
        self.playcurrenttrack()?;

        Ok(())
    }

    fn shufflequeue(&mut self) -> Result<()> {
        Ok(())
    }

    fn repeatqueue(&mut self) -> Result<()> {
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        backend::pause(MPVSOCKET)?;
        self.playing = !self.playing;
        Ok(())
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
        url: String::from("https://www.youtube.com/watch?v=zouSbflXfOo")
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
        currentdurationsecs: 0,
        shuffle: false,
        repeat: RepeatType::None,
        mpv: None, // dont init mpv yet, only start when user starts playing music
        currentlyselectedcolumn: CurrentColumn::Playlists,
        currentlyselectedplaylistidx: 0,
        currentlyselectedtrackidx: 0,
        playlistsstate: ListState::default(),
        tracksstate: ListState::default(),
        queuestate: ListState::default()
    };

    app.firstrun()?;
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
        app.ontick(&counter)?;
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
