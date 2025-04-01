use std::rc::Rc;

use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, LineGauge, List, ListItem, Paragraph},
    Terminal,
};

pub fn mainlayout(area: Rect) -> (Rc<[Rect]>, Rc<[Rect]>, Rc<[Rect]>) {
    let mainlayout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(100),
            Constraint::Min(3)
        ])
        .split(area);

    let upperlayout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30), // playlist
                Constraint::Percentage(45), // tracks
                Constraint::Percentage(25)  // queue
            ])
            .split(mainlayout[0]);

    let lowerlayout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(20),
                    Constraint::Percentage(50),
                    Constraint::Min(10)
                ])
                .split(mainlayout[1]);

    (upperlayout, lowerlayout, mainlayout)
}

pub fn upperview(area: Rect, upperlayout: Rc<[Rect]>) {
    
}

pub fn lowerview(area: Rect, lowerlayout: Rc<[Rect]>) {
    
}

pub fn mainview(area: Rect, mainlayout: Rc<[Rect]>) {
    
}