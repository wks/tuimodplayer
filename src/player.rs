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

use openmpt::module::{metadata::MetadataKey, Module};
use seqlock::SeqLock;

pub struct PlayState {
    pub module_info: ModuleInfo,
    pub moment_state: Arc<SeqLock<MomentState>>,
}

#[derive(Clone)]
pub struct ModuleInfo {
    pub title: String,
    pub n_orders: usize,
    pub n_patterns: usize,
    pub message: Vec<String>,
}

impl ModuleInfo {
    pub fn from_module(module: &mut Module) -> Self {
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

#[derive(Default, Clone, Copy)]
pub struct MomentState {
    pub order: usize,
    pub pattern: usize,
    pub row: usize,
    pub speed: usize,
    pub tempo: usize,
}

impl MomentState {
    pub fn from_module(module: &mut Module) -> Self {
        Self {
            order: module.get_current_order() as _,
            pattern: module.get_current_pattern() as _,
            row: module.get_current_row() as _,
            speed: module.get_current_speed() as _,
            tempo: module.get_current_tempo() as _,
        }
    }
}
