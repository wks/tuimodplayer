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

mod cpal;

use std::time::Duration;

use openmpt::module::Module;

use crate::{control::ModuleControl, player::PlayState};

pub use self::cpal::CpalBackend;

pub trait ModuleProvider: Send {
    /// Get the next module after the current module has been played.
    fn poll_module(&mut self) -> Option<Module>;
}

pub enum BackendEvent {
    StartedPlaying { play_state: PlayState },
    PlayListExhausted,
}

#[derive(Default, Clone, Copy)]
pub struct DecodeStatus {
    pub buffer_samples: usize,
    #[allow(unused)]
    pub decode_time: Duration,
    pub cpu_util: f64,
}

/// The trait for an audio backend.  The main thread owns instances of `Backend`.
pub trait Backend {
    fn start(&mut self);
    fn pause_resume(&mut self);
    fn reload(&mut self);
    fn poll_event(&mut self) -> Option<BackendEvent>;
    fn update_control(&mut self, control: ModuleControl);
    fn read_decode_status(&self) -> DecodeStatus;
}
