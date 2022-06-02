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

use std::sync::{self, mpsc, Arc};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, Stream,
};
use openmpt::module::Module;

use crate::player::{ModuleInfo, MomentState, PlayState};

use super::{Backend, BackendEvent, ControlEvent, ModuleProvider};

pub struct CpalBackend {
    pub host: Host,
    pub device: Device,
    pub stream: Arc<Stream>,
    shared: Arc<CpalBackendShared>,
    paused: bool,
    receiver: mpsc::Receiver<BackendEvent>,
    sender: mpsc::Sender<ControlEvent>,
}

struct CpalBackendShared {
    pub sample_rate: usize,
}

enum CurrentModuleState {
    NotLoaded,
    Loaded {
        module: Module,
        moment_state: Arc<MomentState>,
    },
    Exhausted,
}

struct CpalBackendPrivate {
    shared: Arc<CpalBackendShared>,
    module: CurrentModuleState,
    module_provider: Box<dyn ModuleProvider>,
    stream: sync::Weak<Stream>, // Have to close the loop with Option.
    sender: mpsc::Sender<BackendEvent>,
    receiver: mpsc::Receiver<ControlEvent>,
}

unsafe impl Send for CpalBackendPrivate {}

impl CpalBackendPrivate {
    pub fn on_data_requested(&mut self, data: &mut [f32], _info: &cpal::OutputCallbackInfo) {
        let actual_read_bytes = loop {
            while let Ok(ev) = self.receiver.try_recv() {
                match ev {
                    ControlEvent::Generic(f) => {
                        if let CurrentModuleState::Loaded { ref mut module, .. } = self.module {
                            f(module)
                        }
                    }
                    ControlEvent::Reload => {
                        self.reload();
                    }
                }
            }

            match self.module {
                CurrentModuleState::NotLoaded => {
                    self.reload();
                    continue;
                }
                CurrentModuleState::Exhausted => {
                    self.stop_self();
                    break 0;
                }
                CurrentModuleState::Loaded {
                    ref mut module,
                    ref moment_state,
                } => {
                    let time1 = std::time::Instant::now();
                    let actual_read_frames =
                        module.read_interleaved_float_stereo(self.shared.sample_rate as i32, data);
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
                        moment_state.update_from_module(module);
                        break actual_read_bytes;
                    } else {
                        self.module = CurrentModuleState::NotLoaded;
                        continue;
                    }
                }
            };
        };

        data[actual_read_bytes..].fill(0.0f32);
    }

    fn reload(&mut self) {
        self.module = if let Some(mut module) = self.module_provider.poll_module() {
            let moment_state: Arc<MomentState> = Default::default();
            let play_state = PlayState {
                module_info: ModuleInfo::from_module(&mut module),
                moment_state: moment_state.clone(),
            };
            self.sender
                .send(BackendEvent::StartedPlaying { play_state })
                .unwrap();
            CurrentModuleState::Loaded {
                module,
                moment_state,
            }
        } else {
            self.sender.send(BackendEvent::PlayListExhausted).unwrap();
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

        let config = device
            .supported_output_configs()
            .unwrap()
            .find(|config| {
                let cpal::SampleRate(min_rate) = config.min_sample_rate();
                let cpal::SampleRate(max_rate) = config.max_sample_rate();
                let min_rate = min_rate as usize;
                let max_rate = max_rate as usize;

                config.channels() == CHANNELS
                    && config.sample_format() == SAMPLE_FORMAT
                    && min_rate <= sample_rate
                    && sample_rate <= max_rate
            })
            .expect("No suitable config");

        let config = config.with_sample_rate(cpal::SampleRate(sample_rate as u32));
        log::info!("Using output config: {:?}", config);

        let shared = Arc::new(CpalBackendShared { sample_rate });

        let (ctrl_sender, ctrl_receiver) = mpsc::channel();
        let (be_sender, be_receiver) = mpsc::channel();

        let stream = Arc::new_cyclic(|stream_weak| {
            let mut cpal_writer = CpalBackendPrivate {
                shared: shared.clone(),
                module: CurrentModuleState::NotLoaded,
                module_provider,
                stream: stream_weak.clone(),
                sender: be_sender,
                receiver: ctrl_receiver,
            };

            device
                .build_output_stream(
                    &config.into(),
                    move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                        cpal_writer.on_data_requested(data, info);
                    },
                    |err| panic!("{}", err),
                )
                .unwrap()
        });

        Self {
            host,
            device,
            stream,
            shared,
            paused: false,
            receiver: be_receiver,
            sender: ctrl_sender,
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

    fn reload(&mut self) {
        self.sender.send(ControlEvent::Reload).unwrap();
    }

    fn poll_event(&mut self) -> Option<BackendEvent> {
        match self.receiver.try_recv() {
            Ok(ev) => Some(ev),
            Err(_) => None,
        }
    }

    fn send_event(&mut self, event: super::ControlEvent) {
        self.sender.send(event).unwrap();
    }
}
