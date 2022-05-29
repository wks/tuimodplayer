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

use std::sync::{Arc, self};

use anyhow::Result;

use atomic::{Atomic, Ordering};
use cpal::{
    traits::{HostTrait, StreamTrait},
    Device, Host, Stream,
};
use openmpt::module::Module;
use rodio::{DeviceTrait, OutputStream, OutputStreamHandle, Sink};

use crate::{module_source::ModuleSource, player::PlayState};

pub trait Backend {
    fn start(&mut self);
    fn pause_resume(&mut self);
    fn next(&mut self);
}

pub trait ModuleProvider {
    fn next_module(&mut self) -> Option<Module>;
}

pub struct CpalBackend {
    pub host: Host,
    pub device: Device,
    pub stream: Arc<Stream>,
    shared: Arc<CpalBackendShared>,
    paused: bool,
}

struct CpalBackendShared {
    pub next_requested: Atomic<bool>,
    pub sample_rate: usize,
}

enum CurrentModuleState {
    NotLoaded,
    Loaded(Module),
    Exhausted,
}

struct CpalBackendPrivate {
    shared: Arc<CpalBackendShared>,
    module: CurrentModuleState,
    module_provider: Box<dyn ModuleProvider>,
    stream: sync::Weak<Stream>,  // Have to close the loop with Option.
}

unsafe impl Send for CpalBackendPrivate {}

impl CpalBackendPrivate {
    pub fn on_data_requested(&mut self, data: &mut [f32], _info: &cpal::OutputCallbackInfo) {
        let actual_read_bytes = loop {
            if self.shared.next_requested.swap(false, Ordering::SeqCst) {
                self.load_next();
                continue;
            }

            match self.module {
                CurrentModuleState::NotLoaded => {
                    self.load_next();
                    continue;
                },
                CurrentModuleState::Exhausted => {
                    self.stop_self();
                    break 0;
                },
                CurrentModuleState::Loaded(ref mut module) => {
                    let time1 = std::time::Instant::now();
                    let actual_read_frames = module.read_interleaved_float_stereo(self.shared.sample_rate as i32, data);
                    let time2 = std::time::Instant::now();
                    let elapsed = (time2 - time1).as_micros();
                    let buf_time = actual_read_frames * 1000 * 1000 / self.shared.sample_rate;
                    let actual_read_bytes = actual_read_frames * 2;
                    log::debug!(
                        "data.len: {}, actual_read_bytes: {}, time: {} / {}",
                        data.len(),
                        actual_read_bytes,
                        elapsed,
                        buf_time,
                    );
                    if actual_read_frames > 0 {
                        break actual_read_bytes;
                    } else {
                        self.module = CurrentModuleState::NotLoaded;
                        continue;
                    }
                },
            };
        };

        data[actual_read_bytes..].fill(0.0f32);
    }

    fn load_next(&mut self) {
        self.module = if let Some(module) = self.module_provider.next_module() {
            CurrentModuleState::Loaded(module)
        } else {
            CurrentModuleState::Exhausted
        };
    }

    fn stop_self(&mut self) {
        if let Some(stream) = self.stream.upgrade() {
            stream.pause().unwrap();
        } else {
            panic!("The Stream no longer exists.  Did the main thread quit?");
        }
    }
}

impl CpalBackend {
    pub fn new(sample_rate: usize, module_provider: Box<dyn ModuleProvider>) -> CpalBackend {
        let host = cpal::default_host();

        let device = host.default_output_device().expect("No default device");
        log::info!("Output device: {:?}", device.name());

        const CHANNELS: cpal::ChannelCount = 2;
        const SAMPLE_FORMAT: cpal::SampleFormat = cpal::SampleFormat::F32;

        let config = device.supported_output_configs().unwrap().filter(|config| {
            let cpal::SampleRate(min_rate) = config.min_sample_rate();
            let cpal::SampleRate(max_rate) = config.max_sample_rate();
            let min_rate = min_rate as usize;
            let max_rate = max_rate as usize;

            config.channels() == CHANNELS && config.sample_format() == SAMPLE_FORMAT &&
            min_rate <= sample_rate && sample_rate <= max_rate
        }).next().expect("No suitable config");

        let config = config.with_sample_rate(cpal::SampleRate(sample_rate as u32));
        log::info!("Using output config: {:?}", config);

        let shared = Arc::new(CpalBackendShared {
            next_requested: Atomic::new(false),
            sample_rate,
        });

        let stream = Arc::new_cyclic(|stream_weak| {
            let mut cpal_writer = CpalBackendPrivate {
                shared: shared.clone(),
                module: CurrentModuleState::NotLoaded,
                module_provider,
                stream: stream_weak.clone(),
            };

            let stream = device
                .build_output_stream(
                    &config.into(),
                    move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                        cpal_writer.on_data_requested(data, info);
                    },
                    |err| panic!("{}", err),
                )
                .unwrap();

            stream
        });

        Self {
            host,
            device,
            stream,
            shared,
            paused: false,
        }
    }
}

impl Backend for CpalBackend {
    fn start(&mut self) {
        self.stream.play().unwrap();
    }

    fn pause_resume(&mut self) {
        if self.paused {
            self.stream.play().unwrap();
            self.paused = false;
        } else {
            self.stream.pause().unwrap();
            self.paused = true;
        }
    }

    fn next(&mut self) {
        self.shared.next_requested.store(true, Ordering::SeqCst);
    }
}

pub struct RodioBackend {
    sample_rate: usize,
    _stream: OutputStream,
    handle: OutputStreamHandle,
    _sink: Sink,
}

impl RodioBackend {
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
