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

use openmpt::module::Module;
use std::sync::{Arc, Mutex};

use crate::{
    backend::ModuleProvider,
    module_file::open_module_from_mod_path,
    util::{add_modulo_unsigned, sub_modulo_unsigned},
};

use super::PlayListItem;

pub struct PlayList {
    pub items: Vec<PlayListItem>,
    pub now_playing: Option<usize>,
    pub next_to_play: Option<usize>,
}

impl PlayList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            now_playing: None,
            next_to_play: None,
        }
    }

    pub fn add_item(&mut self, item: PlayListItem) {
        self.items.push(item);
    }

    pub fn poll_module(&mut self) -> Option<Module> {
        if self.next_to_play.is_none() {
            self.goto_next_module(1);
        }

        let mut retries = 0;

        let maybe_module = loop {
            if let Some(index) = self.next_to_play {
                self.now_playing = self.next_to_play.take();

                let item = self.items.get(index).unwrap_or_else(|| {
                    panic!("next_to_play points to non-existing item: {}", index)
                });

                match open_module_from_mod_path(&item.mod_path) {
                    Ok(module) => {
                        break Some(module);
                    }
                    Err(e) => {
                        log::error!(
                            "Error loading module {:?}: {}",
                            item.mod_path.root_path.to_string_lossy(),
                            e
                        );
                    }
                }

                retries += 1;
                if retries >= self.items.len() {
                    break None;
                }

                // Try the next in the playlist.
                self.goto_next_module(1);
            } else {
                log::info!("No more mods to play!");
                break None;
            }
        };

        maybe_module
    }

    pub fn goto_next_module(&mut self, steps: usize) -> bool {
        let maybe_next = if self.items.is_empty() {
            None
        } else if let Some(n) = self.now_playing {
            let len = self.items.len();
            Some(add_modulo_unsigned(n, steps % len, len))
        } else {
            Some(0)
        };

        self.next_to_play = maybe_next;
        maybe_next.is_some()
    }

    pub fn goto_previous_module(&mut self, steps: usize) -> bool {
        let maybe_next = if self.items.is_empty() {
            None
        } else if let Some(n) = self.now_playing {
            let len = self.items.len();
            Some(sub_modulo_unsigned(n, steps % len, len))
        } else {
            let len = self.items.len();
            Some(len - 1)
        };

        self.next_to_play = maybe_next;
        maybe_next.is_some()
    }
}

pub struct PlayListModuleProvider {
    playlist: Arc<Mutex<PlayList>>,
}

impl PlayListModuleProvider {
    pub fn new(playlist: Arc<Mutex<PlayList>>) -> Self {
        Self { playlist }
    }
}

impl ModuleProvider for PlayListModuleProvider {
    fn poll_module(&mut self) -> Option<Module> {
        self.playlist.lock().unwrap().poll_module()
    }
}
