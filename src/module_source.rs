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

use atomic::{Atomic, Ordering};
use openmpt::module::Module;
use rodio::Source;

#[derive(Default)]
pub struct PlayState {
    pub order: Atomic<usize>,
    pub pattern: Atomic<usize>,
    pub row: Atomic<usize>,
    pub n_rows: Atomic<usize>,
    pub speed: Atomic<usize>,
    pub tempo: Atomic<usize>,
}

pub struct ModuleSource {
    module: Module,
    play_state: Arc<PlayState>,
    sample_rate: usize,
    buf: Vec<f32>,
    cursor: usize,
    limit: usize,
}

unsafe impl Send for ModuleSource {}

/// The default sample rate.
///
/// libopenmpt recommends 48000 because
/// "practically all audio equipment and file formats use 48000Hz nowadays".
pub const DEFAULT_SAMPLE_RATE: usize = 48000;

/// Minimum sample rate supported by libopenmpt.
pub const MIN_SAMPLE_RATE: usize = 8000;

/// Maximum sample rate supported by libopenmpt.
pub const MAX_SAMPLE_RATE: usize = 192000;

impl ModuleSource {
    pub fn new(module: Module, play_state: Arc<PlayState>, sample_rate: usize) -> Self {
        Self {
            module,
            play_state,
            sample_rate,
            buf: vec![0.0f32; 128 * 2],
            cursor: 0,
            limit: 0,
        }
    }

    fn update_play_state(&mut self) {
        let play_state = &self.play_state;
        let module = &mut self.module;

        let order = module.get_current_order();
        let pattern = module.get_current_pattern();
        let row = module.get_current_row();

        let mut pattern_obj = module.get_pattern_by_number(pattern).unwrap();
        let n_rows = pattern_obj.get_num_rows();

        let speed = module.get_current_speed();
        let tempo = module.get_current_tempo();

        play_state.order.store(order as usize, Ordering::SeqCst);
        play_state.pattern.store(pattern as usize, Ordering::SeqCst);
        play_state.row.store(row as usize, Ordering::SeqCst);
        play_state.n_rows.store(n_rows as usize, Ordering::SeqCst);
        play_state.speed.store(speed as usize, Ordering::SeqCst);
        play_state.tempo.store(tempo as usize, Ordering::SeqCst);
    }
}

impl Iterator for ModuleSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor >= self.limit {
            let frames_read = self
                .module
                .read_interleaved_float_stereo(self.sample_rate() as i32, &mut self.buf);
            if frames_read == 0 {
                return None;
            }
            self.cursor = 0;
            self.limit = frames_read * self.channels() as usize;

            self.update_play_state();
        }

        let data = self.buf[self.cursor];
        self.cursor += 1;
        Some(data)
    }
}

impl rodio::Source for ModuleSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate as u32
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
