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

mod cpal;
mod rodio;

use openmpt::module::Module;

use crate::player::PlayState;

pub use self::cpal::CpalBackend;
pub use self::rodio::RodioBackend;

pub trait ModuleProvider {
    fn next_module(&mut self) -> Option<Module>;
}

pub enum BackendEvent {
    StartedPlaying {
        play_state: PlayState,
    },
    PlayListExhausted,
}

pub trait Backend {
    fn start(&mut self);
    fn pause_resume(&mut self);
    fn next(&mut self);
    fn poll_event(&mut self) -> Option<BackendEvent>;
}
