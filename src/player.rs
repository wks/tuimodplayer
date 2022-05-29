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

use atomic::Atomic;
use openmpt::module::{metadata::MetadataKey, Module};

pub struct PlayState {
    pub module_info: ModuleInfo,
    pub moment_state: Arc<MomentState>,
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

#[derive(Default)]
pub struct MomentState {
    pub version: Atomic<usize>,
    pub order: Atomic<usize>,
    pub pattern: Atomic<usize>,
    pub row: Atomic<usize>,
    pub speed: Atomic<usize>,
    pub tempo: Atomic<usize>,
}

pub struct MomentStateCopy {
    pub order: usize,
    pub pattern: usize,
    pub row: usize,
    pub speed: usize,
    pub tempo: usize,
}

impl MomentState {
    fn store<T, U>(field: &Atomic<T>, value: U)
    where
        T: 'static + Copy,
        U: Copy + num_traits::AsPrimitive<T>,
    {
        field.store(value.as_(), atomic::Ordering::Relaxed);
    }

    fn load<T>(field: &Atomic<T>) -> T
    where
        T: 'static + Copy,
    {
        field.load(atomic::Ordering::Relaxed)
    }

    pub fn update_from_module(&self, module: &mut Module) {
        self.version.fetch_add(1, atomic::Ordering::SeqCst);
        Self::store(&self.order, module.get_current_order());
        Self::store(&self.pattern, module.get_current_pattern());
        Self::store(&self.row, module.get_current_row());
        Self::store(&self.speed, module.get_current_speed());
        Self::store(&self.tempo, module.get_current_tempo());
        self.version.fetch_add(1, atomic::Ordering::SeqCst);
    }

    pub fn load_atomic(&self) -> MomentStateCopy {
        loop {
            let version1 = self.version.load(atomic::Ordering::SeqCst);
            if version1 % 2 == 1 {
                continue;
            }
            let result = MomentStateCopy {
                order: Self::load(&self.order),
                pattern: Self::load(&self.pattern),
                row: Self::load(&self.row),
                speed: Self::load(&self.speed),
                tempo: Self::load(&self.tempo),
            };
            let version2 = self.version.load(atomic::Ordering::SeqCst);

            if version2 != version1 {
                continue;
            }

            break result;
        }
    }
}
