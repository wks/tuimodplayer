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

use crate::sink::RodioState;
use crate::ui::run_ui;

use openmpt::module::{metadata::MetadataKey, Module};

use anyhow::Result;

pub struct AppState {
    pub mod_info: Option<ModuleInfo>,
    pub play_state: Arc<PlayState>,
    pub rodio_state: RodioState,
    pub playlist: Vec<PlayListItem>,
    pub cur_module: usize,
}

impl AppState {
    pub fn start_playing(&mut self) {
        while self.cur_module < self.playlist.len() {
            let item = &self.playlist[self.cur_module];
            if let Err(e) = open_module_from_mod_path(&item.mod_path).and_then(|mut module| {
                self.mod_info = Some(ModuleInfo::from_module(&mut module));
                self.rodio_state
                    .play_module(module, self.play_state.clone())
            }) {
                log::info!(
                    "Cannot play {}: {}",
                    item.mod_path.root_path.to_string_lossy(),
                    e
                );
                self.cur_module += 1;
                continue;
            }
            break;
        }

        log::info!("No more mod to play");
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

pub fn run(options: Options) -> Result<()> {
    let play_state = Arc::new(PlayState::default());
    let rodio_state = RodioState::new(options.sample_rate)?;

    let playlist = if let Some(file_path) = options.file_path {
        playlist::load_from_path(&file_path)
    } else {
        vec![]
    };

    let mut app_state = AppState {
        mod_info: None,
        play_state,
        rodio_state,
        playlist,
        cur_module: 0,
    };

    app_state.start_playing();

    run_ui(&mut app_state)?;

    Ok(())
}
