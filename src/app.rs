// Copyright 2022 Kunshan Wang
//
// This file is part of TUIModPlayer.  TUIModPlayer is free software: you can redistribute it
// and/or modify it under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// TUIModPlayer is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See
// the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with TUIModPlayer. If
// not, see <https://www.gnu.org/licenses/>.

use std::sync::Arc;

use crate::options::Options;
use crate::player::PlayState;
use crate::playlist::{PlayList, PlayListModuleProvider};

use crate::backend::{Backend, BackendEvent, CpalBackend};
use crate::ui::run_ui;

use anyhow::Result;

pub struct AppState {
    pub play_state: Option<PlayState>,
    pub backend: Box<dyn Backend>,
    pub playlist: Arc<PlayList>,
    pub cur_module: usize,
}

impl AppState {
    pub fn start_playing(&mut self) {
        self.backend.start();
    }

    pub fn next(&mut self) {
        self.backend.next();
    }

    pub fn pause_resume(&mut self) {
        self.backend.pause_resume();
    }

    pub fn handle_backend_events(&mut self) {
        while let Some(be_ev) = self.backend.poll_event() {
            match be_ev {
                BackendEvent::StartedPlaying { play_state } => {
                    self.play_state = Some(play_state);
                }
                BackendEvent::PlayListExhausted => {
                    self.play_state = None;
                }
            }
        }
    }
}

pub fn run(options: Options) -> Result<()> {
    let mut playlist = PlayList::new();

    for path in options.paths.iter() {
        playlist.load_from_path(path);
    }

    let playlist = Arc::new(playlist);
    let module_provider = Box::new(PlayListModuleProvider::new(playlist.clone()));

    let backend: Box<dyn Backend> =
        Box::new(CpalBackend::new(options.sample_rate, module_provider));

    let mut app_state = AppState {
        play_state: None,
        backend,
        playlist,
        cur_module: 0,
    };

    app_state.start_playing();

    run_ui(&mut app_state)?;

    Ok(())
}
