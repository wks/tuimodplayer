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

use crate::{module_file::open_module_file, options::Options};

use crate::module_source::{ModuleSource, PlayState};
use crate::ui::run_ui;

use openmpt::module::{metadata::MetadataKey, Module};

use anyhow::{Context, Result};
use rodio::{OutputStream, OutputStreamHandle, Sink};

pub struct AppState {
    pub mod_info: ModuleInfo,
    pub play_state: Arc<PlayState>,
    pub rodio_state: RodioState,
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

pub struct RodioState {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    _sink: Sink,
}

impl RodioState {
    fn new() -> Result<Self> {
        let (_stream, handle) = rodio::OutputStream::try_default()?;
        let _sink = rodio::Sink::try_new(&handle)?;
        Ok(Self {
            _stream,
            handle,
            _sink,
        })
    }
    fn play_module(&mut self, module: Module, play_state: Arc<PlayState>) -> Result<()> {
        let module_source = ModuleSource::new(module, play_state);
        self.handle.play_raw(module_source)?;

        //sink.append(module_source);
        // sink.sleep_until_end();

        Ok(())
    }
}

pub fn run(options: Options) -> Result<()> {
    let file_path = options.file_path;

    let mut module = open_module_file(file_path.clone())
        .with_context(|| format!("Cannot open file {}", file_path))?;

    module.set_repeat_count(-1);

    let play_state = Arc::new(PlayState::default());
    let rodio_state = RodioState::new()?;

    let mut app_state = AppState {
        mod_info: ModuleInfo::from_module(&mut module),
        play_state: play_state.clone(),
        rodio_state,
    };

    app_state.rodio_state.play_module(module, play_state)?;

    run_ui(&mut app_state)?;

    Ok(())
}
