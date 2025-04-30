// frontend.rs
// handles drawing, etc

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style}, text::{Line, Text}, widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph}, Frame
};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::Duration;

use crate::{backend::{self, Backend}, models::{Playlist, RepeatMode}};
use crate::models::{CurrentColumn, SONGINFOPERCENT, Track, VERSION};

#[derive(Clone, Debug)]
pub struct PopupState {
    pub onscreen: bool,
    pub title: String,
    pub header: String,
    pub message: Vec<String>,
    pub dangerous: bool,
}

pub struct App {
    pub version: String,
    pub popup: PopupState,
}

impl App {
    pub fn new(version: String) -> Self {
        Self {
            version,
            popup: PopupState {
                onscreen: false,
                title: String::new(),
                header: String::new(),
                message: Vec::new(),
                dangerous: false,
            },
        }
    }
}

pub fn runfrontend(mut backend: Backend) -> Result<()> {
    // --- setup terminal ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backendtui = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backendtui)?;

    // --- initialize ui state ---
    let mut running = true;
    let mut app = App::new(VERSION.to_string());

    // --- main loop ---
    let mut counter: u8 = 0;
    while running {
        let elapsed = backend::get::elapsedduration(&backend)?;
        let total = backend::get::totalduration(&backend)?;
        let isplaying = *backend::get::playingstate(&backend);
        let selectedcolumn = backend::get::selectedcolumn(&backend);
        let mut playliststate = backend::get::playliststate(&backend).clone();
        if playliststate.selected().is_none() && !backend::get::playlistsempty(&backend) {
            playliststate.select(Some(0));
        }

        let selectedplaylistidx = playliststate.selected();
        let trackstodisplay = if let Some(playlistidx) = selectedplaylistidx {
            backend::get::playlisttracks(&backend, playlistidx).unwrap_or_default()
        } else {
            Vec::new()
        };
        
        let mut trackstate = backend.selection.trackstate.clone();
        if trackstate.selected().is_none() && !trackstodisplay.is_empty() {
            trackstate.select(Some(0));
        }
        
        let mut queuestate = backend.selection.queuestate.clone();
        if queuestate.selected().is_none() && !backend::get::queueempty(&backend) {
            queuestate.select(Some(0));
        }
        
        terminal.draw(|frame| {
            let area = frame.area();
            let (playlists, tracks, queue, controls, songname, progressbar, credits) =
                construct(area, &isplaying);

            let playlistscont = getplaylistscont(backend::get::playlists(&backend), selectedcolumn == &CurrentColumn::Playlists);
            let trackscont = gettrackscont(&trackstodisplay, selectedcolumn == &CurrentColumn::Tracks);
            let queuecont = getqueuecont(backend::get::queue(&backend), selectedcolumn == &CurrentColumn::Queue);
            let controlscont = getcontrolscontent(&isplaying);
            let songinfocont = backend::get::currentsong(&backend)
                .map(|track| getsonginfocont(track, 
                                            *backend::get::shufflestate(&backend), 
                                            backend::get::repeatstate(&backend).clone()))
                .unwrap_or_else(|| getsonginfocont(&Track::default(), false, RepeatMode::None));
            let progressbarcont = getprogressbarcont(&elapsed, &total);
            let creditscont = getcreditscont(VERSION);
            
            renderpopup(&app, frame);
            frame.render_stateful_widget(playlistscont, playlists, &mut playliststate);
            frame.render_stateful_widget(trackscont, tracks, &mut trackstate);
            frame.render_stateful_widget(queuecont, queue, &mut queuestate);
            frame.render_widget(controlscont, controls);
            frame.render_widget(songinfocont, songname);
            frame.render_widget(progressbarcont, progressbar);
            frame.render_widget(creditscont, credits);
        })?;

        // handle input
        let popupopen = app.popup.onscreen;
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let Err(e) = handleinput(&mut backend, &mut app, &mut running, key.code, popupopen) {
                        newpopup(&mut app, 
                            "Error".to_string(),
                            vec![format!("Input error: {}", e)],
                            true
                        );
                    }
                }
            }
        }

        // handle periodic updates (e.g., track progress)
        counter = (counter + 1) % 4;
        if counter == 0 {
            // TODO: update currenttime or check track end via backend
        }
    }

    // --- cleanup ---
    let _ = backend.shutdown();
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

fn handleinput(
    backend: &mut Backend,
    app: &mut App,
    running: &mut bool,
    key: KeyCode,
    popupopen: bool,
) -> Result<()> {
    if popupopen.to_owned() {
        if key == KeyCode::Char('q') || key == KeyCode::Esc {
            closepopup(app);
        }
        return Ok(());
    }

    // handle input based on app state
    match key {
        KeyCode::Char('q') => *running = false,
        KeyCode::Char(' ') => backend::set::playpause(backend)?, // toggle play/pause
        KeyCode::Char('>') => backend::set::next(backend)?,
        KeyCode::Char('<') => backend::set::prev(backend)?, // TODO: restart song when < 10 secs
        KeyCode::Char('s') => backend::set::toggleshuffle(backend),
        KeyCode::Char('r') => backend::set::cyclerepeat(backend),
        KeyCode::Up => backend::set::prevrow(backend),
        KeyCode::Down => backend::set::nextrow(backend),
        KeyCode::Left => backend::set::prevcolumn(backend),
        KeyCode::Right => backend::set::nextcolumn(backend),
        KeyCode::Enter => { /* ... play selected track or playlist ... */ }
        _ => {}
    }
    Ok(())
}


/// main constructor
/// 
/// # arguments
/// * 'area' - the area to split up into individual areas
/// 
/// # returns
/// * a tuple of six rects:
///     * playlists
///     * tracks
///     * queue
///     * controls
///     * songinfo
///     * progressbar
pub fn construct(area: Rect, isplaying: &bool) -> (Rect, Rect, Rect, Rect, Rect, Rect, Rect) {
    let verticalchunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(100),// tracks views
                Constraint::Min(3),         // controls/info
                Constraint::Length(1)       // credits
            ])
            .split(area);

    let toplayout = verticalchunks[0];
    let bottomlayout = verticalchunks[1];

    let topchunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(2),            // playlists
                Constraint::Fill(5),            // tracks
                Constraint::Fill(2)             // queue
            ])
            .split(toplayout);

    let bottomchunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                if isplaying.to_owned() {[
                    Constraint::Length(21),                     // controls
                    Constraint::Percentage(SONGINFOPERCENT),    // song name
                    Constraint::Min(20)                         // progress bar
                ]} else {[
                    Constraint::Length(20),                     // controls
                    Constraint::Percentage(SONGINFOPERCENT),    // song name
                    Constraint::Min(20)                         // progress bar
                ]}
            )
            .split(bottomlayout);

    let playlists = topchunks[0];
    let tracks = topchunks[1];
    let queue = topchunks[2];

    let controls = bottomchunks[0];
    let songname = bottomchunks[1];
    let progressbar = bottomchunks[2];

    let credits = verticalchunks[2];

    (playlists, tracks, queue, controls, songname, progressbar, credits)
}

fn getcontrolscontent(isplaying: &bool) -> Paragraph {
    let text = if !isplaying.to_owned() {
        String::from("[<<] [ play ] [>>]")
    } else {
        String::from("[<<] [ pause ] [>>]")
    };

    Paragraph::new(text)
        .block(
            Block::new()
                .title(" controls ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Magenta))
        .alignment(ratatui::layout::Alignment::Center)
}

fn getplaylistscont(playlists: &Vec<Playlist>, infocus: bool) -> List {
    // gets the list of playlists
    let playlistitems: Vec<ListItem> = playlists
        .iter()
        .map(|p| ListItem::new(format!(" {}", p.name.as_str())))
        .collect();

    let playlistslist = List::new(playlistitems)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(
                    if infocus {
                        Style::default().fg(Color::Magenta)
                    } else {
                        Style::default()
                    }
                )
                .border_type(
                    if infocus {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    }
                )
                .title(" playlists ")
        )
        .highlight_style(
            if infocus {
                Style::default().bg(Color::Magenta).fg(Color::Black)
            } else {
                Style::default().bg(Color::LightMagenta).fg(Color::White)
            }
        )
        .highlight_symbol("> ");

    playlistslist
}

fn gettrackscont(tracks: &Vec<Track>, infocus: bool) -> List {
    // gets the list of tracks
    let trackitems: Vec<ListItem> = tracks
        .iter()
        .map(|t| ListItem::new(format!(" {} - {}", t.title.as_str(), t.artist.as_str())))
        .collect();

    let trackslist = List::new(trackitems)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(
                    if infocus {
                        Style::default().fg(Color::Magenta)
                    } else {
                        Style::default()
                    }
                )
                .border_type(
                    if infocus {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    }
                )
                .title(" tracks ")
        )
        .highlight_style(
            if infocus {
                Style::default().bg(Color::Magenta).fg(Color::Black)
            } else {
                Style::default().bg(Color::LightMagenta).fg(Color::White)
            }
        )
        .highlight_symbol("> ");

    trackslist
}

fn getqueuecont(tracks: &Vec<Track>, infocus: bool) -> List {
    // gets the list of tracks
    let trackitems: Vec<ListItem> = tracks
        .iter()
        .map(|t| ListItem::new(String::from(t.title.as_str())))
        .collect();

    let trackslist = List::new(trackitems)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(
                    if infocus {
                        Style::default().fg(Color::Magenta)
                    } else {
                        Style::default()
                    }
                )
                .border_type(
                    if infocus {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    }
                )
                .title(" queue ")
        )
        .highlight_style(
            if infocus {
                Style::default().bg(Color::Magenta).fg(Color::Black)
            } else {
                Style::default().bg(Color::LightMagenta).fg(Color::White)
            }
        )
        .highlight_symbol("> ");

    trackslist
}


// These functions have been moved to models.rs

fn getprogressbarcont(currenttime: &u32, totaltime: &u32) -> Gauge<'static> {
    let current = currenttime.to_owned();
    let total = totaltime.to_owned();
    // gets the progressbar 
    let currentprogress: String = getprettyduration(current);
    let totalprogress: String = getprettyduration(total);
    let currentprogressratio;
    if total == 0 {
        currentprogressratio = 0f64;
    } else {
        currentprogressratio = current as f64/total as f64;
    }

    Gauge::default()
        .block(
            Block::default().title(format!(" {}/{} ", currentprogress, totalprogress))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
        )
        .style(Style::default().fg(Color::Magenta))
        .gauge_style(Style::default().fg(Color::LightMagenta))
        .label("")
        .ratio(currentprogressratio)
}

fn getprettyduration(secs: u32) -> String {
    let minutes = secs / 60;
    let seconds = secs % 60;
    format!("{}:{:02}", minutes, seconds)
}

fn getcreditscont(version: &str) -> Block {
    // gets the credits
    Block::new()
        .title_top(format!("mpvlayer ── v{} ", version))
        .title_top(Line::from(" https://github.com/ellipticobj/mpvlayer").right_aligned())
        .border_type(BorderType::Rounded)
        .borders(Borders::TOP)
}

pub fn getsonginfocont(track: &Track, shuffle: bool, repeat: RepeatMode) -> ratatui::widgets::Paragraph<'static> {
    use ratatui::{style::{Color, Style}, text::Line, widgets::{Block, BorderType, Borders, Paragraph}};

    // --- create display text based on track info ---
    let displaytext: String = if track.artist.is_empty() {
        String::from(format!(" {} ", track.title))
    } else if track.title.is_empty() {
        String::from(format!(" Unknown Track - {} ", track.artist))
    } else {
        String::from(format!(" {} - {} ", track.title, track.artist))
    };

    // --- get controls state string ---
    let controlsstatestring = getcontrolsstate(shuffle, repeat);
    let controlsstateline = Line::from(format!(" {} ", controlsstatestring)).right_aligned();

    // --- build the final Paragraph ---
    Paragraph::new(displaytext)
        .block(
            Block::default()
                .title_top(" currently playing ")
                .title_top(controlsstateline)
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::Magenta))
        .alignment(ratatui::layout::Alignment::Left)
}

pub fn getcontrolsstate(shuffle: bool, repeat: RepeatMode) -> String {
    // gets the state of shuffle and repeat
    let mut controls: Vec<String> = Vec::new();
    if shuffle {
        controls.push(String::from("shuffle on ─")); // extra dash so the text stays still call me a sigma
    } else {
        controls.push(String::from("shuffle off "));
    }

    match repeat {
        RepeatMode::None => controls.push(String::from(" repeat off")),
        RepeatMode::One => controls.push(String::from(" repeat one")),
        RepeatMode::All => controls.push(String::from(" repeat all"))
    }

    controls.join("──")
}

/// centers a rect
/// 
/// # arguments
/// * `rect` - the rect to center
/// * `area` - the area to center in
/// 
/// # returns
/// * the centered rect
pub fn centerrect(rect: Rect, area: Rect) -> Rect {
    let x = (area.width - rect.width) / 2;
    let y = (area.height - rect.height) / 2;
    Rect::new(x, y, rect.width, rect.height)
}

pub fn renderpopup(app: &App, frame: &mut Frame) {
    if app.popup.onscreen {
        let area = frame.area();
        let popuparea = centerrect(Rect::new(0, 0, 40, 10), area);
        let popupcont = Paragraph::new(Text::from(app.popup.message.clone().iter().map(|s| Line::from(s.clone())).collect::<Vec<Line>>()))
            .block(Block::default().title(app.popup.title.clone()).borders(Borders::ALL))
            .style(Style::default().fg(Color::Magenta))
            .alignment(ratatui::layout::Alignment::Left);
        frame.render_widget(popupcont, popuparea);
    }
}

pub fn newpopup(app: &mut App, title: String, message: Vec<String>, dangerous: bool) {
    app.popup.onscreen = true;
    app.popup.title = title;
    app.popup.message = message;
    app.popup.dangerous = dangerous;
}

pub fn closepopup(app: &mut App) {
    app.popup.onscreen = false;
    app.popup.title = String::from("");
    app.popup.message = Vec::new();
    app.popup.dangerous = false;
}
