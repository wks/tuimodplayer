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

use cpal::{Device, Host};
use openmpt::module::Module;
use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::{module_source::ModuleSource, player::PlayState};

struct CpalState {
    pub host: Host,
    pub device: Device,
}

pub struct RodioState {
    sample_rate: usize,
    _stream: OutputStream,
    handle: OutputStreamHandle,
    _sink: Sink,
}

impl RodioState {
    pub fn new(sample_rate: usize) -> Result<Self> {
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
