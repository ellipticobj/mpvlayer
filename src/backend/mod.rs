// mod.rs
// backend

pub mod get;
pub mod set;

use crate::models::*;
use anyhow::Result;
use std::process::Child;

pub struct Backend {
    pub(crate) state: AppState,
    pub(crate) selection: SelectionState,
    pub(crate) mpvprocess: Option<Child>,
}

impl Backend {
    pub fn new() -> Self {
        // initialize a new liststate
        let mut playliststate = ratatui::widgets::ListState::default();
        playliststate.select(Some(0));

        // initialize track state
        let mut trackstate = ratatui::widgets::ListState::default();
        trackstate.select(Some(0));

        // initialize queue state
        let mut queuestate = ratatui::widgets::ListState::default();
        queuestate.select(Some(0));

        // initialize the backend
        let backend = Backend {
            state: AppState {
                playlists: Vec::new(),
                player: PlayerState {
                    isplaying: false,
                    currenttime: 0,
                    queuestate: QueueState {
                        queue: Vec::new(),
                        history: Vec::new(),
                    },
                    repeatstate: RepeatState {
                        repeatmode: RepeatMode::None,
                        originalqueue: vec![],
                    },
                    shufflestate: ShuffleState {
                        shuffle: false,
                        originalqueue: vec![],
                    },
                },
            },
            selection: SelectionState {
                selectedcolumn: CurrentColumn::Playlists, // playlists column selected on startup
                playliststate,
                trackstate,
                queuestate,
            },
            mpvprocess: None,
        };

        backend
    }

    pub fn shutdown(&mut self) -> Result<()> {
        if let Some(mut process) = self.mpvprocess.take() {
            let _ = process.kill();
        }

        Ok(())
    }
}
