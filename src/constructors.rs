use ratatui::{
    layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style}, text::Line, widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph}, Frame
};

use crate::consts::{App, CurrentColumn, Track};

static CONTROLSLENGTH: u16 = 21;
static SONGINFOPERCENT: u16 = 70;

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
pub fn construct(area: Rect) -> (Rect, Rect, Rect, Rect, Rect, Rect, Rect) {
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
                Constraint::Fill(1),            // playlists
                Constraint::Fill(3),            // tracks
                Constraint::Fill(1)             // queue
            ])
            .split(toplayout);

    let bottomchunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(CONTROLSLENGTH),         // controls
                Constraint::Percentage(SONGINFOPERCENT),    // song name
                Constraint::Min(20)                         // progress bar
            ])
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

fn getcontrolscont(app: &App) -> Paragraph {
    // gets controls
    let controls;
    if app.playing {
        controls = "[<<] [ pause ] [>>]";
    } else {
        controls = "[<<] [ play ] [>>]";
    }

    Paragraph::new(controls)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" controls ")
        )
        .style(Style::default().fg(Color::Magenta))
        .alignment(ratatui::layout::Alignment::Center)
}

fn getplaylistscont(playlists: &Vec<crate::consts::Playlist>, infocus: bool) -> List {
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
        .highlight_style(Style::default().bg(Color::LightMagenta))
        .highlight_symbol("> ");

    playlistslist
}

fn gettrackscont(tracks: &Vec<crate::consts::Track>, infocus: bool) -> List {
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
        .highlight_style(Style::default().bg(Color::LightMagenta))
        .highlight_symbol("> ");

    trackslist
}

fn getqueuecont(queue: &Vec<Track>, infocus: bool) -> List<'static> {
    // gets the play queue
    let queueitems: Vec<ListItem> = queue
        .iter()
        .map(|t| ListItem::new(format!(" {}", t.title.as_str())))
        .collect();

    let queuelist = List::new(queueitems)
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
        .highlight_style(Style::default().bg(Color::LightMagenta))
        .highlight_symbol("> ");

    queuelist
}

fn getcontrolsstate(shuffle: bool, repeat: crate::consts::RepeatType) -> String {
    // gets the state of shuffle and repeat 
    let mut controls: Vec<String> = Vec::new();
    if shuffle {
        controls.push(String::from("shuffle on ─")); // extra dash so the text stays still call me a sigma
    } else {
        controls.push(String::from("shuffle off "));
    }

    match repeat {
        crate::consts::RepeatType::None => controls.push(String::from(" repeat off")),
        crate::consts::RepeatType::One => controls.push(String::from(" repeat one")),
        crate::consts::RepeatType::All => controls.push(String::from(" repeat all"))
    }

    controls.join("──")
}

fn getsonginfocont(queue: &Vec<Track>, currentqueueidx: u32, shuffle: bool, repeat: crate::consts::RepeatType) -> Paragraph<'static> {
    // gets currently playing song
    let displaytext: String;

    // --- check if the index points to a valid track ---
    let trackidx = currentqueueidx as usize;
    if !queue.is_empty() && trackidx < queue.len() {
        // --- valid track ---
        let currenttrack = &queue[trackidx];

        displaytext = if !currenttrack.artist.is_empty() {
            format!(" {} - {}", currenttrack.artist, currenttrack.title)
        } else {
            format!(" {}", currenttrack.title)
        };
    } else {
        // --- no valid track (empty queue or invalid index) ---
        displaytext = " no song playing ".to_string();
    }

    // --- get controls state string ---
    let controlsstatestring = getcontrolsstate(shuffle, repeat);
    let controlsstateline = ratatui::text::Line::from(format!(" {} ", controlsstatestring)).right_aligned();

    // --- build the final Paragraph ---
    Paragraph::new(displaytext)
        .block(
            Block::default()
                .title_top(" currently playing ")
                .title_top(controlsstateline)
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::Magenta))
        .alignment(ratatui::layout::Alignment::Left)
}

fn getprettyduration(secs: u32) -> String {
    // changes duration from seconds to m:ss
    let minutes = secs / 60;
    let seconds = secs % 60;

    format!("{}:{:02}", minutes, seconds)
}

fn getprogressbar(currentprogresssecs: u32, totalsecs: u32) -> Gauge<'static> {
    // gets the progressbar 
    let currentprogress: String = getprettyduration(currentprogresssecs);
    let totalprogress: String = getprettyduration(totalsecs);
    let currentprogressratio;
    if totalsecs == 0 {
        currentprogressratio = 0f64;
    } else {
        currentprogressratio = currentprogresssecs as f64/totalsecs as f64;
    }

    Gauge::default()
        .block(Block::default().title(format!(" {}/{} ", currentprogress, totalprogress)).borders(Borders::ALL))
        .style(Style::default().fg(Color::Magenta))
        .gauge_style(Style::default().fg(Color::LightMagenta))
        .label("")
        .ratio(currentprogressratio)
}

fn getcreditscont(version: &str) -> Block<'static> {
    // gets the credits
    Block::new()
        .title_top(format!("mpvlayer ── v{} ", version))
        .title_top(Line::from(" complain at https://github.com/ellipticobj/mpvlayer").right_aligned())
        .borders(Borders::TOP)
}

/// renders the main view
/// 
/// # arguments
/// * `app` - mutable reference to the app state
/// * `frame` - mutable reference to the frame to render on
/// * `areas` - tuple of rectangular areas for different UI components
/// 
/// # returns
/// * nothing
pub fn rendermainview(app: &mut App, frame: &mut Frame, areas: (Rect, Rect, Rect, Rect, Rect, Rect, Rect)) {
    let (playlists, tracks, queue, controls, songinfo, progressbar, credits) = areas;

    let playlistscont = getplaylistscont(&app.playlists, app.currentcolumn == CurrentColumn::Playlists);
    frame.render_stateful_widget(playlistscont, playlists, &mut app.playliststate);
    
    if !app.playlists.is_empty() && (app.playliststate.selected().unwrap_or(0)) < app.playlists.len() {
        let trackscont = gettrackscont(&app.playlists[app.playliststate.selected().unwrap_or(0)].tracks, app.currentcolumn == CurrentColumn::Tracks);
        frame.render_stateful_widget(trackscont, tracks, &mut app.tracksstate);
    } else {
        frame.render_widget(Block::default().borders(Borders::ALL).title(" tracks "), tracks);
    }

    let controlscont = getcontrolscont(app);
    frame.render_widget(controlscont, controls);

    let songinfocont = getsonginfocont(&app.queue, app.currentqueueidx, app.shuffle, app.repeat);
    frame.render_widget(songinfocont, songinfo);

    let creditscont = getcreditscont(&app.version);
    frame.render_widget(creditscont, credits);

    let progressbarcont;
    // if queue is not empty and current index is valid and current duration is valid
    if !app.queue.is_empty() && (app.currentqueueidx as usize) < app.queue.len() && app.currentdurationsecs <= app.queue[app.currentqueueidx as usize].duration {
        progressbarcont = getprogressbar(app.currentdurationsecs, app.queue[app.currentqueueidx as usize].duration);
    } else {
        progressbarcont = getprogressbar(0, 0);
    }
    frame.render_widget(progressbarcont, progressbar);

    let queuecont = getqueuecont(&app.queue, app.currentcolumn == CurrentColumn::Queue);
    frame.render_stateful_widget(queuecont, queue, &mut app.queuestate);
}
