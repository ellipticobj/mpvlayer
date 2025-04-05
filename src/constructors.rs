use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
    text::Span
};

use crate::consts::{App, Queue};

pub fn construct(area: Rect) -> (Rect, Rect, Rect, Rect, Rect, Rect) {
    let verticalchunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(100),
                Constraint::Min(3)
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
                Constraint::Length(21),         // controls
                Constraint::Percentage(70),     // song name
                Constraint::Min(20)             // progress bar
            ])
            .split(bottomlayout);

    let playlists = topchunks[0];
    let tracks = topchunks[1];
    let queue = topchunks[2];

    let controls = bottomchunks[0];
    let songname = bottomchunks[1];
    let progressbar = bottomchunks[2];

    (playlists, tracks, queue, controls, songname, progressbar)

}
fn getplaylistscont(playlists: &Vec<crate::consts::Playlist>) -> List {
    let playlistitems: Vec<ListItem> = playlists
        .iter()
        .map(|p| ListItem::new(format!(" {}", p.name.as_str())))
        .collect();

    let playlistslist = List::new(playlistitems)
        .block(Block::default().borders(Borders::ALL).title(" playlists "))
        .highlight_style(Style::default().bg(Color::LightMagenta))
        .highlight_symbol("> ");

    playlistslist
}

fn gettrackscont(tracks: &Vec<crate::consts::Track>) -> List {
    let trackitems: Vec<ListItem> = tracks
        .iter()
        .map(|t| ListItem::new(format!(" {}", t.title.as_str())))
        .collect();

    let trackslist = List::new(trackitems)
        .block(Block::default().borders(Borders::ALL).title(" tracks "))
        .highlight_style(Style::default().bg(Color::LightMagenta))
        .highlight_symbol("> ");

    trackslist
}

fn getcontrolscont(app: &App) -> Paragraph {
    let controls;
    if app.playing {
        controls = "[<<] [ pause ] [>>]";
    } else {
        controls = "[<<] [ play ] [>>]";
    }

    Paragraph::new(controls)
        .block(Block::default().borders(Borders::ALL).title(" controls "))
        .style(Style::default().fg(Color::Magenta))
        .alignment(ratatui::layout::Alignment::Center)
}

fn getqueuecont(queue: Queue) -> List<'static> {
    let queueitems: Vec<ListItem> = queue
        .queue
        .iter()
        .map(|t| ListItem::new(format!(" {}", t.title.as_str())))
        .collect();

    let queuelist = List::new(queueitems)
        .block(Block::default().borders(Borders::ALL).title(" queue "))
        .highlight_style(Style::default().bg(Color::LightMagenta))
        .highlight_symbol("> ");

    queuelist
}

fn getsonginfocont(queue: &Queue, currentqueueidx: u32) -> Paragraph<'static> {
    if queue.queue.is_empty() || queue.queue.len() <= currentqueueidx as usize {
        return Paragraph::new(" no song playing ")
            .block(Block::default().title(" song ").borders(Borders::ALL))
            .style(Style::default().fg(Color::Magenta))
            .alignment(ratatui::layout::Alignment::Left);
    }

    let currenttrack = &queue.queue[currentqueueidx as usize];

    Paragraph::new(format!(" {} - {}", currenttrack.artist, currenttrack.title))
        .block(Block::default().title(" song ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Magenta))
        .alignment(ratatui::layout::Alignment::Left)
}

fn getprogressbar() -> Gauge<'static> {
    let currentprogress: f64 = 5f64;
    let totalprogress: f64 = 10f64;

    Gauge::default()
        .block(Block::default().title(" progress ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Magenta))
        .gauge_style(Style::default().fg(Color::LightMagenta))
        .label(format!(" {}/{} ", currentprogress, totalprogress))
        .ratio(currentprogress/totalprogress)
}

pub fn rendermainview(app: &mut App, frame: &mut Frame, areas: (Rect, Rect, Rect, Rect, Rect, Rect)) {
    let (playlists, tracks, queue, controls, songinfo, progressbar) = areas;

    let playlistscont = getplaylistscont(&app.playlists);
    frame.render_stateful_widget(playlistscont, playlists, &mut app.playlistsstate);
    
    if !app.playlists.is_empty() && (app.currentlyselectedplaylistidx as usize) < app.playlists.len() {
        let trackscont = gettrackscont(&app.playlists[app.currentlyselectedplaylistidx as usize].tracks);
        frame.render_stateful_widget(trackscont, tracks, &mut app.tracksstate);
    } else {
        frame.render_widget(Block::default().borders(Borders::ALL).title(" tracks "), tracks);
    }

    let controlscont = getcontrolscont(app);
    frame.render_widget(controlscont, controls);

    let songinfocont = getsonginfocont(&app.queue.clone(), app.currentqueueidx);
    frame.render_widget(songinfocont, songinfo);

    let progressbarcont = getprogressbar();
    frame.render_widget(progressbarcont, progressbar);

    frame.render_widget(Block::default().title(" queue ").borders(Borders::ALL), queue);
}