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

use anyhow::Result;

use atomic::Ordering;

use openmpt::module::Module;
use rodio::{OutputStream, OutputStreamHandle, Sink};

use super::{Backend, ModuleProvider};

use rodio::Source;

use crate::player::PlayState;

pub struct RodioBackend {
    sample_rate: usize,
    _stream: OutputStream,
    handle: OutputStreamHandle,
    _sink: Sink,
}

impl RodioBackend {
    pub fn new(sample_rate: usize, _module_provider: Box<dyn ModuleProvider>) -> Result<Self> {
        let (_stream, handle) = rodio::OutputStream::try_default()?;
        let _sink = rodio::Sink::try_new(&handle)?;
        Ok(Self {
            sample_rate,
            _stream,
            handle,
            _sink,
        })
    }
    pub fn play_module(&mut self, module: Module, play_state: Arc<PlayState>) -> Result<()> {
        let module_source = ModuleSource::new(module, play_state, self.sample_rate);
        self.handle.play_raw(module_source)?;

        //sink.append(module_source);
        // sink.sleep_until_end();

        Ok(())
    }

    pub fn sample_rate(&self) -> usize {
        self.sample_rate
    }
}

impl Backend for RodioBackend {
    fn start(&mut self) {
        todo!()
    }

    fn pause_resume(&mut self) {
        todo!()
    }

    fn next(&mut self) {
        todo!()
    }

    fn poll_event(&mut self) -> Option<super::BackendEvent> {
        todo!()
    }
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

//         play_state.order.store(order as usize, Ordering::SeqCst);
//         play_state.pattern.store(pattern as usize, Ordering::SeqCst);
//         play_state.row.store(row as usize, Ordering::SeqCst);
//         play_state.n_rows.store(n_rows as usize, Ordering::SeqCst);
//         play_state.speed.store(speed as usize, Ordering::SeqCst);
//         play_state.tempo.store(tempo as usize, Ordering::SeqCst);
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
