// Copyright 2022, 2024, 2025 Kunshan Wang
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

use std::sync::{Arc, Mutex};

use crate::control::ModuleControl;

use crate::options::Options;
use crate::player::PlayState;
use crate::playlist::{PlayList, PlayListModuleProvider};

use crate::backend::{Backend, BackendEvent, CpalBackend};
use crate::ui::color_scheme::ColorScheme;
use crate::ui::run_ui;

use anyhow::Result;

#[derive(Default)]
pub enum UiMode {
    #[default]
    Normal,
    Filter,
}

pub struct AppState {
    pub options: Options,
    pub play_state: Option<PlayState>,
    pub backend: Box<dyn Backend>,
    pub playlist: Arc<Mutex<PlayList>>,
    pub control: ModuleControl,
    pub ui_mode: UiMode,
    pub color_scheme: ColorScheme,
}

impl AppState {
    pub fn start_playing(&mut self) {
        self.backend.start();
    }

    pub fn next(&mut self) {
        self.playlist.lock().unwrap().goto_next_module(1);
        self.backend.reload();
    }

    pub fn prev(&mut self) {
        self.playlist.lock().unwrap().goto_previous_module(1);
        self.backend.reload();
    }

    pub fn next10(&mut self) {
        self.playlist.lock().unwrap().goto_next_module(10);
        self.backend.reload();
    }

    pub fn prev10(&mut self) {
        self.playlist.lock().unwrap().goto_previous_module(10);
        self.backend.reload();
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

    fn send_apply_mod_settings_event(&mut self) {
        let control_clone = self.control.clone();
        self.backend.update_control(control_clone);
    }

    pub fn tempo_down(&mut self) {
        self.control.tempo.dec();
        self.send_apply_mod_settings_event();
    }

    pub fn tempo_up(&mut self) {
        self.control.tempo.inc();
        self.send_apply_mod_settings_event();
    }

    pub fn pitch_down(&mut self) {
        self.control.pitch.dec();
        self.send_apply_mod_settings_event();
    }

    pub fn pitch_up(&mut self) {
        self.control.pitch.inc();
        self.send_apply_mod_settings_event();
    }

    pub fn gain_down(&mut self) {
        self.control.gain.dec();
        self.send_apply_mod_settings_event();
    }

    pub fn gain_up(&mut self) {
        self.control.gain.inc();
        self.send_apply_mod_settings_event();
    }

    pub fn stereo_separation_down(&mut self) {
        self.control.stereo_separation.dec();
        self.send_apply_mod_settings_event();
    }

    pub fn stereo_separation_up(&mut self) {
        self.control.stereo_separation.inc();
        self.send_apply_mod_settings_event();
    }

    pub fn filter_taps_down(&mut self) {
        self.control.filter_taps.dec();
        self.send_apply_mod_settings_event();
    }

    pub fn filter_taps_up(&mut self) {
        self.control.filter_taps.inc();
        self.send_apply_mod_settings_event();
    }

    pub fn volume_ramping_down(&mut self) {
        self.control.volume_ramping.dec();
        self.send_apply_mod_settings_event();
    }

    pub fn volume_ramping_up(&mut self) {
        self.control.volume_ramping.inc();
        self.send_apply_mod_settings_event();
    }

    pub fn toggle_repeat(&mut self) {
        self.control.repeat = !self.control.repeat;
        self.send_apply_mod_settings_event();
    }
}

pub fn run(options: Options) -> Result<()> {
    let mut playlist = PlayList::new();

    log::info!("Loading from {} root paths...", options.paths.len());
    for path in options.paths.iter() {
        crate::playlist::load_from_path(&mut playlist, path, options.deep_archive_search);
    }

    log::info!("Shuffling playlist...");
    if options.shuffle {
        playlist.shuffle();
    }

    let playlist = Arc::new(Mutex::new(playlist));
    let module_provider = Box::new(PlayListModuleProvider::new(playlist.clone()));

    let control = ModuleControl::default();

    let backend: Box<dyn Backend> = Box::new(CpalBackend::new(
        options.sample_rate,
        module_provider,
        control.clone(),
    ));

    let mut app_state = AppState {
        options,
        play_state: None,
        backend,
        playlist,
        control,
        ui_mode: Default::default(),
        color_scheme: Default::default(),
    };

    app_state.start_playing();

    run_ui(&mut app_state)?;

    Ok(())
}
