use std::{
    clone, error::Error, io, process::{
        Child, Command, Stdio
    }, rc::Rc, time::{Duration, Instant}
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

pub mod ytdlp;

struct App {
    running: bool,
    version: String,
    mpv: Option<Child>,
    playing: bool,
    current: CurrentlyPlaying,
    playlists: Vec<Playlist>,
    selectedplaylistidx: Option<usize>,
    selectedtrackidx: Option<usize>
}

#[derive(Clone)]
struct Track {
    title: String,
    artist: String,
    url: String,
    duration: String,
}

struct Playlist {
    title: Option<String>,
    url: String,
    tracks: Vec<Track>
}

struct CurrentlyPlaying {
    track: Option<Track>,
    playlist: Option<Playlist>,
    progress: Option<u16>,
}

impl App {
    fn new() -> App {
        App {
            running: true,
            version: String::from("v0.0.1"),
            mpv: None,
            playing: false,
            current: CurrentlyPlaying { track: None, playlist: None, progress: None },
            playlists: vec![Playlist {
                title: Some(String::from("test playlist")),
                url: String::from("testurl"),
                tracks: vec![Track {
                    title: String::from("track 1"),
                    artist: String::from("artist 1"),
                    url: String::from("https://www.youtube.com/watch?v=eDshx6Rg9Hs"),
                    duration: String::from("2:45"),
                }],
            }],
            selectedplaylistidx: Some(0),
            selectedtrackidx: Some(0),
        }
    }

    fn ontick(&mut self) {
        
    }

    fn onkey(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.running = false;
            }
            KeyCode::Char(' ') => {
                
                self.playing = !self.playing;
            }
            KeyCode::Char('j') => {
                // skip track
            }
            KeyCode::Char('l') => {
                // unskip track
            }
            KeyCode::Left => {
                // select next track
                if let Some(playlist_index) = self.selectedplaylistidx {
                    if let Some(track_index) = self.selectedtrackidx {
                        let playlist = &self.playlists[playlist_index];
                        if track_index < playlist.tracks.len() - 1 {
                            self.selectedtrackidx = Some(track_index + 1);
                        }
                    }
                }
            }
            KeyCode::Right => {
                // select previous track
                if let Some(track_index) = self.selectedtrackidx {
                    if track_index > 0 {
                        self.selectedtrackidx = Some(track_index - 1);
                    }
                }
            }
            KeyCode::Up => {
                // select next playlist
                if let Some(playlist_index) = self.selectedplaylistidx {
                    if playlist_index < self.playlists.len() - 1 {
                        self.selectedplaylistidx = Some(playlist_index + 1);
                    }
                }
            }
            KeyCode::Down => {
                // select previous playlist
                if let Some(playlist_index) = self.selectedplaylistidx {
                    if playlist_index > 0 {
                        self.selectedplaylistidx = Some(playlist_index - 1);
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(playlist_index) = self.selectedplaylistidx {
                    if let Some(track_index) = self.selectedtrackidx {
                        let track = &self.playlists[playlist_index].tracks[track_index];
                        self.mpv = startmpv(&track.url).ok();
                    }
                }
            }
            _ => {}
        }
    }
}

fn startmpv(url: &str) -> Result<std::process::Child> {
    let mpv = Command::new("mpv")
        .arg("--input-file=-") // read commands
        .arg("--idle") // keep running when theres nothing else
        .arg("--no-terminal") // quiet
        .arg("--no-video")
        .arg("")
        .arg(url)
        .stdin(Stdio::piped())
        .spawn()?;

    Ok(mpv)
}

fn createtrack(url: &str) -> Result<Track> {
    let (duration, artist, title) = ytdlp::getmetadata(url);

    Ok(Track{title, artist, url: String::from(url), duration})
}

fn createplaylist(playlist: Vec<Track>) -> Vec<ListItem<'static>> {
    playlist
        .iter()
        .map(|track| {
            let text = format!(" {} - {} ({})", track.title, track.artist, track.duration);
            ListItem::new(text)
        })
        .collect()
}

fn createlayout(area: Rect, progressbarwidth: u16) -> (Rc<[Rect]>, Rc<[Rect]>, Rc<[Rect]>) {
    // top and bottom
    let mainlayout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),   // top
            Constraint::Length(3) // player at the bottom
        ])
        .split(area);

    // top: playlist and inner
    let toplayout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20), // playlist
            Constraint::Percentage(70)  // inner
        ])
        .split(mainlayout[0]);

    // bottom: player
    let playerlayout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20), // controls
            Constraint::Percentage(70), // info
            Constraint::Min(50),
        ])
        .split(mainlayout[1]);

    (toplayout, playerlayout, mainlayout)
}

fn playlistview(listitems: Vec<ListItem>) -> List {
    List::new(listitems)
                .block(Block::default().title(" playlist ").borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow))
                .highlight_symbol(">> ")
}

fn playerview(app: &App, track: &Track, progress: u16, progressbarwidth: u16) -> (Block<'static>, Paragraph<'static>, Paragraph<'static>, LineGauge<'static>) {
    let playpausetext = if app.playing { "pause" } else { "play" };

    let controls = format!(" [<<] [{}] [>>]", playpausetext);

    let trackinfo = Paragraph::new(format!("{} - {}", track.title, track.artist))
        .alignment(ratatui::layout::Alignment::Left)
        .block(Block::default().borders(Borders::NONE));

    let controlspara = Paragraph::new(controls).alignment(ratatui::layout::Alignment::Left);

    let progressratio = progress as f64 / progressbarwidth as f64;
    let progressbar = LineGauge::default()
        .block(Block::default().borders(Borders::NONE))
        .filled_style(Style::default().fg(Color::White))
        .label(format!("{}/{}", progress, 100))
        .line_set(ratatui::symbols::line::DOUBLE)
        .ratio(progressratio);

    let playerblock = Block::default()
        .borders(Borders::ALL)
        .title(" currently playing ");
    
    (playerblock, trackinfo, controlspara, progressbar)
}

fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tickrate: Duration,
) -> Result<(), Box<dyn Error>> {
    let mut lasttick = Instant::now();
    let progressbarwidth = 20;
    // hardcoded for now
    let progress = 0;
    

    loop {
        terminal.draw(|f| {
            // main window
            let area = f.area();
            let (toplayout, playerlayout, mainlayout) = createlayout(area, progressbarwidth);
            
            let track = Track{title: String::from("track 1"), artist: String::from("artist 1"), url: String::from("https://www.youtube.com/watch?v=eDshx6Rg9Hs"), duration: String::from("2:45")};
            let playlist = vec![track.clone()];
            let listitems: Vec<ListItem> = createplaylist(playlist);
            let list = playlistview(listitems);

            f.render_widget(list, toplayout[0]);

            f.render_widget(
                Paragraph::new("inner 1")
                    .block(Block::new().borders(Borders::ALL)),
                toplayout[1],
            );

            let (playerblock, trackinfo, controlspara, progressbar) = 
                playerview(&app, &track, progress, progressbarwidth);
            
            
            f.render_widget(playerblock, playerlayout[0].union(playerlayout[1]).union(playerlayout[2]));

            f.render_widget(controlspara, playerlayout[0].inner(Margin {vertical: 1, horizontal: 1}));
            f.render_widget(trackinfo, playerlayout[1].inner(Margin {vertical: 1, horizontal: 1}));
            f.render_widget(progressbar, playerlayout[2].inner(Margin {vertical: 1, horizontal: 1}));
        })?;

        let timeout = tickrate
            .checked_sub(lasttick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.onkey(key.code);
                }
            }
        }

        if lasttick.elapsed() >= tickrate {
            app.ontick();
            lasttick = Instant::now();
        }

        if !app.running {
            return Ok(());
        }
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout, 
        EnterAlternateScreen, 
        // EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // create app and run it
    let tickrate = Duration::from_millis(250);
    let app = App::new();
    let res = run(&mut terminal, app, tickrate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        // DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
