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

use crate::module_file::open_module_from_mod_path;
use crate::options::Options;
use crate::player::PlayState;
use crate::playlist::{self, PlayListItem};

use crate::backend::{Backend, CpalBackend, ModuleProvider, RodioBackend};
use crate::ui::run_ui;

use openmpt::module::{metadata::MetadataKey, Module};

use anyhow::Result;

pub struct AppState {
    pub mod_info: Option<ModuleInfo>,
    pub play_state: Arc<PlayState>,
    backend: Box<dyn Backend>,
    pub playlist: Arc<Vec<PlayListItem>>,
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
}

pub struct ModuleInfo {
    pub title: String,
    pub n_orders: usize,
    pub n_patterns: usize,
    pub message: Vec<String>,
}

impl ModuleInfo {
    fn from_module(module: &mut Module) -> Self {
        let title = module
            .get_metadata(MetadataKey::ModuleTitle)
            .unwrap_or_else(|| "(no title)".to_string());
        let n_orders = module.get_num_orders() as usize;
        let n_patterns = module.get_num_patterns() as usize;
        let message = {
            let n_instruments = module.get_num_instruments();
            if n_instruments != 0 {
                (0..n_instruments)
                    .map(|i| module.get_instrument_name(i))
                    .collect::<Vec<_>>()
            } else {
                let n_samples = module.get_num_samples();
                (0..n_samples)
                    .map(|i| module.get_sample_name(i))
                    .collect::<Vec<_>>()
            }
        };
        Self {
            title,
            n_orders,
            n_patterns,
            message,
        }
    }
}

struct VecModuleProvider {
    vector: Arc<Vec<PlayListItem>>,
    cursor: usize,
}

impl VecModuleProvider {
    pub fn new(vector: Arc<Vec<PlayListItem>>) -> Self {
        Self { vector, cursor: 0 }
    }
}

impl ModuleProvider for VecModuleProvider {
    fn next_module(&mut self) -> Option<Module> {
        if self.cursor < self.vector.len() {
            let item = &self.vector[self.cursor];
            self.cursor += 1;
            match open_module_from_mod_path(&item.mod_path) {
                Ok(module) => Some(module),
                Err(e) => {
                    log::error!(
                        "Error loading module {:?}: {}",
                        item.mod_path.root_path.to_string_lossy(),
                        e
                    );
                    None
                }
            }
        } else {
            log::info!("No more mods to play!");
            None
        }
    }
}

pub fn run(options: Options) -> Result<()> {
    let play_state = Arc::new(PlayState::default());

    let playlist = if let Some(file_path) = options.file_path {
        playlist::load_from_path(&file_path)
    } else {
        vec![]
    };

    let playlist = Arc::new(playlist);
    let module_provider = Box::new(VecModuleProvider::new(playlist.clone()));

    let backend: Box<dyn Backend> = if options.cpal {
        Box::new(CpalBackend::new(options.sample_rate, module_provider))
    } else {
        Box::new(RodioBackend::new(options.sample_rate, module_provider)?)
    };

    let mut app_state = AppState {
        mod_info: None,
        play_state,
        backend,
        playlist,
        cur_module: 0,
    };

    app_state.start_playing();

    run_ui(&mut app_state)?;

    Ok(())
}
