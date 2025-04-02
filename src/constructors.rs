use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
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
                Constraint::Length(20),         // controls
                Constraint::Percentage(70),     // song name
                Constraint::Min(10)             // progress bar
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

pub fn drawmainview(app: &App, frame: &mut Frame, areas: (Rect, Rect, Rect, Rect, Rect, Rect)) {
    let (playlists, tracks, queue, controls, songinfo, progressbar) = areas;
    
    frame.render_widget(Block::default().title(" playlists ").borders(Borders::ALL), playlists);
    frame.render_widget(Block::default().title(" tracks ").borders(Borders::ALL), tracks);
    frame.render_widget(Block::default().title(" queue ").borders(Borders::ALL), queue);
    frame.render_widget(Block::default().title(" controls ").borders(Borders::ALL), controls);
    frame.render_widget(Block::default().title(" song info ").borders(Borders::ALL), songinfo);
    frame.render_widget(Block::default().title(" progress bar ").borders(Borders::ALL), progressbar);
}