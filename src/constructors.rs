use ratatui::{
    layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style}, widgets::{Block, Borders, List, ListItem, Paragraph}, Frame
};

use crate::consts::App;

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



pub fn rendermainview(app: &mut App, frame: &mut Frame, areas: (Rect, Rect, Rect, Rect, Rect, Rect)) {
    let (playlists, tracks, queue, controls, songinfo, progressbar) = areas;

    let playlistscont = getplaylistscont(&app.playlists);
    let trackscont = gettrackscont(&app.playlists[app.currentlyselectedplaylistidx as usize].tracks);
    
    frame.render_stateful_widget(playlistscont, playlists, &mut app.playlistsstate);
    frame.render_stateful_widget(trackscont, tracks, &mut app.tracksstate);

    let controlscont = getcontrolscont(app);
    frame.render_widget(controlscont, controls);

    frame.render_widget(Block::default().title(" queue ").borders(Borders::ALL), queue);
    frame.render_widget(Block::default().title(" controls ").borders(Borders::ALL), controls);
    frame.render_widget(Block::default().title(" song info ").borders(Borders::ALL), songinfo);
    frame.render_widget(Block::default().title(" progress bar ").borders(Borders::ALL), progressbar);
}