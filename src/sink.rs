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

use std::sync::{Arc, Mutex};

use anyhow::Result;

use cpal::{traits::{HostTrait, StreamTrait}, Device, Host, Stream};
use openmpt::module::Module;
use rodio::{DeviceTrait, OutputStream, OutputStreamHandle, Sink};

use crate::{module_source::ModuleSource, player::PlayState};

pub struct CpalState {
    pub host: Host,
    pub device: Device,
    pub stream: Stream,
    shared: Arc<CpalStateShared>,
}

struct CpalStateShared {
    pub module: Mutex<Option<Module>>,
    pub pause_requested: bool,
}

struct CpalWriter {
    shared: Arc<CpalStateShared>,
}

unsafe impl Send for CpalWriter {}

impl CpalWriter {
    pub fn new(shared: Arc<CpalStateShared>) -> Self {
        Self {
            shared,
        }
    }
    pub fn write_data(&mut self, data: &mut [f32], _info: &cpal::OutputCallbackInfo) {
        let mut maybe_module = self.shared.module.lock().unwrap();
        if let Some(ref mut module) = maybe_module.as_mut() {
            let mut buf = Vec::with_capacity(data.len());
            buf.resize(data.len(), 0.0f32);
            let actual_read = module.read_interleaved_float_stereo(44100 as i32, &mut buf);
            log::info!("data.len: {}, buf.capa: {}, buf.len: {}, actual_read: {}",
                data.len(), buf.capacity(), buf.len(), actual_read);
            data.copy_from_slice(&buf);
        } else {
            data.fill_with(|| { 0.0f32 })
        }
    }
}

impl CpalState {
    pub fn new() -> CpalState {
        let host = cpal::default_host();

        let device = host.default_output_device().expect("No default device");
        log::info!("Output device: {:?}", device.name());

        let shared = Arc::new(CpalStateShared {
            module: Mutex::new(None),
            pause_requested: false,
        });

        let config = device.default_output_config().unwrap();
        log::info!("Default output config: {:?}", config);

        let mut cpal_writer = CpalWriter::new(shared.clone());

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                cpal_writer.write_data(data, info);
            },
            |err| { panic!("{}", err) },
        ).unwrap();

        Self {
            host,
            device,
            stream,
            shared,
        }
    }

    pub fn play_module(&mut self, module: Module, play_state: Arc<PlayState>) -> Result<()> {
        {
            let mut maybe_module = self.shared.module.lock().unwrap();
            *maybe_module = Some(module);
        }

        self.stream.play()?;

        Ok(())
    }
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
