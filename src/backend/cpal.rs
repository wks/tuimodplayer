// Copyright 2022, 2024, 2025 Kunshan Wang
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

use std::{
    sync::{self, Arc, Condvar, Mutex, mpsc},
    time::{Duration, Instant},
};

use cpal::{
    Device, Host, Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use openmpt::module::Module;
use seqlock::SeqLock;

use crate::{
    control::ModuleControl,
    module_file::apply_mod_settings,
    player::{ModuleInfo, MomentState, PlayState},
};

use super::{Backend, BackendEvent, DecodeStatus, ModuleProvider};

/// CPAL backend.  This struct is owned by the main thread.
pub struct CpalBackend {
    #[allow(unused)]
    pub host: Host,
    #[allow(unused)]
    pub device: Device,
    pub stream: Arc<Stream>,
    shared: Arc<CpalBackendShared>,
    paused: bool,
    receiver: mpsc::Receiver<BackendEvent>,
}

struct CpalBackendShared {
    pub sample_rate: usize,
    pub decode_status: SeqLock<DecodeStatus>,
    pub module_and_provider: Mutex<ModuleAndProvider>,
    pub need_service_cond: Condvar,
}

unsafe impl Send for CpalBackendShared {}
unsafe impl Sync for CpalBackendShared {}

enum CurrentModuleState {
    NotLoaded,
    Loaded {
        module: Module,
        moment_state: Arc<SeqLock<MomentState>>,
    },
    Exhausted,
}

struct ModuleAndProvider {
    pub module: CurrentModuleState,
    pub provider: Box<dyn ModuleProvider>,
    pub control: ModuleControl,
    pub on_event: Box<dyn Fn(BackendEvent) + Send>,
}

const CHANNELS: usize = 2;

impl ModuleAndProvider {
    pub fn reload(&mut self) {
        self.module = if let Some(mut module) = self.provider.poll_module() {
            apply_mod_settings(&mut module, &self.control);
            let moment_state: Arc<SeqLock<MomentState>> = Default::default();
            let play_state = PlayState {
                module_info: ModuleInfo::from_module(&mut module),
                moment_state: moment_state.clone(),
            };
            (self.on_event)(BackendEvent::StartedPlaying { play_state });
            CurrentModuleState::Loaded {
                module,
                moment_state,
            }
        } else {
            (self.on_event)(BackendEvent::PlayListExhausted);
            CurrentModuleState::Exhausted
        };
    }

    pub fn update_control(&mut self, control: ModuleControl) {
        self.control = control;
        if let CurrentModuleState::Loaded { ref mut module, .. } = self.module {
            apply_mod_settings(module, &self.control);
        }
    }
}

struct CpalWaiter {
    shared: Arc<CpalBackendShared>,
}

unsafe impl Send for CpalWaiter {}

impl CpalWaiter {
    pub fn run(self) {
        let mut map = self.shared.module_and_provider.lock().unwrap();
        loop {
            match map.module {
                CurrentModuleState::NotLoaded => {
                    map.reload();
                }
                _ => {
                    map = self.shared.need_service_cond.wait(map).unwrap();
                }
            }
        }
    }
}

struct CpalBackendPrivate {
    shared: Arc<CpalBackendShared>,
    stream: sync::Weak<Stream>, // Have to close the loop with Option.
}

unsafe impl Send for CpalBackendPrivate {}

enum ModuleReadResult {
    WouldBlock,
    NotLoaded,
    Exhausted,
    Read { frames: usize, elapsed: Duration },
}

impl CpalBackendPrivate {
    pub fn on_data_requested(&mut self, data: &mut [f32], _info: &cpal::OutputCallbackInfo) {
        let result = self.read_as_much_as_possible_and_dont_block(data);

        let actual_read_samples = if let ModuleReadResult::Read { frames, .. } = result {
            frames * CHANNELS
        } else {
            0
        };

        data[actual_read_samples..].fill(0f32);

        match result {
            ModuleReadResult::WouldBlock => {
                log::debug!("Would block! Not reading from module.");
            }
            ModuleReadResult::NotLoaded => {}
            ModuleReadResult::Exhausted => {
                self.stop_self();
            }
            ModuleReadResult::Read { frames, elapsed } => {
                self.update_statistics(data.len(), frames, elapsed);
            }
        }
    }

    fn read_as_much_as_possible_and_dont_block(&mut self, buf: &mut [f32]) -> ModuleReadResult {
        match self.shared.module_and_provider.try_lock() {
            Err(_) => ModuleReadResult::WouldBlock,
            Ok(mut map) => match map.module {
                CurrentModuleState::NotLoaded => ModuleReadResult::NotLoaded,
                CurrentModuleState::Exhausted => ModuleReadResult::Exhausted,
                CurrentModuleState::Loaded {
                    ref mut module,
                    ref moment_state,
                } => {
                    let before_reading = Instant::now();
                    let actual_read_frames =
                        module.read_interleaved_float_stereo(self.shared.sample_rate as i32, buf);
                    let elapsed = before_reading.elapsed();

                    if actual_read_frames == 0 {
                        map.module = CurrentModuleState::NotLoaded;
                        self.shared.need_service_cond.notify_all();
                    } else {
                        let new_moment_state = MomentState::from_module(module);
                        {
                            let mut moment_state = moment_state.lock_write();
                            *moment_state = new_moment_state;
                        }
                    }

                    ModuleReadResult::Read {
                        frames: actual_read_frames,
                        elapsed,
                    }
                }
            },
        }
    }

    fn stop_self(&mut self) {
        if let Some(stream) = self.stream.upgrade() {
            stream.pause().unwrap();
        } else {
            panic!("The Stream no longer exists.  Did the main thread quit?");
        }
    }

    fn update_statistics(
        &mut self,
        buffer_samples: usize,
        read_frames: usize,
        decode_time: Duration,
    ) {
        let decode_micros = decode_time.as_micros();
        let buf_time_micros = read_frames * 1000 * 1000 / self.shared.sample_rate;
        let read_samples = read_frames * CHANNELS;
        let cpu_util = if read_frames == 0 {
            0f64
        } else {
            // Equal to elapsed_micros / buf_time_micros, but more precise.
            decode_time.as_nanos() as f64 * self.shared.sample_rate as f64
                / (read_frames as f64 * 1_000_000_000_f64)
        };
        log::trace!(
            "buf: {}, read: {}, time: {}µs / {}µs, cpu: {}%",
            buffer_samples,
            read_samples,
            decode_micros,
            buf_time_micros,
            cpu_util * 100.0,
        );
        {
            let mut decode_status = self.shared.decode_status.lock_write();
            *decode_status = DecodeStatus {
                buffer_samples,
                decode_time,
                cpu_util,
            };
        }
    }
}

impl CpalBackend {
    pub fn new(
        sample_rate: usize,
        module_provider: Box<dyn ModuleProvider>,
        control: ModuleControl,
    ) -> CpalBackend {
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

        let (be_sender, be_receiver) = mpsc::channel();

        let shared = Arc::new(CpalBackendShared {
            sample_rate,
            decode_status: Default::default(),
            module_and_provider: Mutex::new(ModuleAndProvider {
                module: CurrentModuleState::NotLoaded,
                provider: module_provider,
                control,
                on_event: Box::new(move |ev| {
                    be_sender.send(ev).unwrap();
                }),
            }),
            need_service_cond: Condvar::new(),
        });

        let waiter = CpalWaiter {
            shared: shared.clone(),
        };

        std::thread::Builder::new()
            .name("CpalWaiter".to_string())
            .spawn(move || {
                waiter.run();
            })
            .unwrap();

        let stream = Arc::new_cyclic(|stream_weak| {
            let mut cpal_writer = CpalBackendPrivate {
                shared: shared.clone(),
                stream: stream_weak.clone(),
            };

            device
                .build_output_stream(
                    &config.into(),
                    move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                        cpal_writer.on_data_requested(data, info);
                    },
                    |err| panic!("{}", err),
                    None,
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
        let mut map = self.shared.module_and_provider.lock().unwrap();
        map.reload();
    }

    fn poll_event(&mut self) -> Option<BackendEvent> {
        self.receiver.try_recv().ok()
    }

    fn update_control(&mut self, control: super::ModuleControl) {
        let mut map = self.shared.module_and_provider.lock().unwrap();
        map.update_control(control);
    }

    fn read_decode_status(&self) -> DecodeStatus {
        self.shared.decode_status.read()
    }
}
