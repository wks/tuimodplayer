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
use rand::prelude::SliceRandom;
use std::sync::{Arc, Mutex};

use crate::{
    backend::ModuleProvider,
    module_file::open_module_from_mod_path,
    util::{add_modulo_unsigned, sub_modulo_unsigned, IsSomeAnd},
};

use super::PlayListItem;

pub struct PlayList {
    pub items: Vec<PlayListItem>,
    pub now_playing_in_items: Option<usize>,
    pub now_playing_in_view: Option<usize>,
    pub next_to_play: Option<usize>,
    view: ListView,
}

enum ListView {
    Direct,
    Filtered {
        filter_string: String,
        filtered_items: Vec<usize>,
    },
}

enum MoveDir {
    Forward,
    Backward,
}

impl PlayList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            now_playing_in_items: None,
            now_playing_in_view: None,
            next_to_play: None,
            view: ListView::Direct,
        }
    }

    pub fn len(&self) -> usize {
        match &self.view {
            ListView::Direct => self.items.len(),
            ListView::Filtered { filtered_items, .. } => filtered_items.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self.view {
            ListView::Direct => self.items.is_empty(),
            ListView::Filtered { filtered_items, .. } => filtered_items.is_empty(),
        }
    }

    pub fn get_item(&self, i: usize) -> Option<&PlayListItem> {
        match &self.view {
            ListView::Direct => self.items.get(i),
            ListView::Filtered { filtered_items, .. } => filtered_items.get(i).map(|j| {
                self.items.get(*j).unwrap_or_else(|| {
                    panic!(
                        "filtered_items[{}] = {} is outside the range of items[]: [{}, {})",
                        i,
                        *j,
                        0,
                        self.items.len()
                    )
                })
            }),
        }
    }

    fn view_index_to_items_index(&self, view_index: usize) -> usize {
        match &self.view {
            ListView::Direct => view_index,
            ListView::Filtered { filtered_items, .. } => filtered_items[view_index],
        }
    }

    pub fn get_filter_string(&self) -> Option<String> {
        match &self.view {
            ListView::Direct => None,
            ListView::Filtered { filter_string, .. } => Some(filter_string.clone()),
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
                self.now_playing_in_view = self.next_to_play.take();
                self.now_playing_in_items = self.now_playing_in_view.map(|view_index| self.view_index_to_items_index(view_index));

                let item = self.get_item(index).unwrap_or_else(|| {
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
                if retries >= self.len() {
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

    fn move_rel(&mut self, steps: usize, dir: MoveDir) -> bool {
        let maybe_next = if self.is_empty() {
            None
        } else if let Some(n) = self.now_playing_in_view {
            let len = self.len();
            let result = match dir {
                MoveDir::Forward => add_modulo_unsigned(n, steps % len, len),
                MoveDir::Backward => sub_modulo_unsigned(n, steps % len, len),
            };
            Some(result)
        } else {
            let result = match dir {
                MoveDir::Forward => 0,
                MoveDir::Backward => self.len() - 1,
            };
            Some(result)
        };

        self.next_to_play = maybe_next;
        maybe_next.is_some()
    }

    pub fn goto_next_module(&mut self, steps: usize) -> bool {
        self.move_rel(steps, MoveDir::Forward)
    }

    pub fn goto_previous_module(&mut self, steps: usize) -> bool {
        self.move_rel(steps, MoveDir::Backward)
    }

    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        self.items.shuffle(&mut rng);
    }

    pub fn update_filter(&mut self, string: String) {
        if string.is_empty() {
            self.view = ListView::Direct;
            self.now_playing_in_view = self.now_playing_in_items;
        } else {
            let filter_string = string;
            let lower_string = filter_string.to_lowercase();
            let case_insensitive_contains =
                |string2: &String| string2.to_lowercase().contains(&lower_string);
            let filtered_items = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(i, item)| {
                    if case_insensitive_contains(&item.mod_path.display_name())
                        || item
                            .metadata
                            .is_some_and2(|metadata| case_insensitive_contains(&metadata.title))
                    {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let new_now_playing_in_view = self.now_playing_in_items.and_then(|items_index| {
                filtered_items.iter().position(|item| *item == items_index)
            });
            self.view = ListView::Filtered {
                filter_string,
                filtered_items,
            };
            self.now_playing_in_view = new_now_playing_in_view;
        }
    }

    pub fn update_filter_push(&mut self, ch: char) {
        match &mut self.view {
            ListView::Direct => self.update_filter(ch.to_string()),
            ListView::Filtered { filter_string, .. } => {
                let mut new_filter_string = std::mem::take(filter_string);
                new_filter_string.push(ch);
                self.update_filter(new_filter_string);
            }
        }
    }

    pub fn update_filter_pop(&mut self) {
        match &mut self.view {
            ListView::Direct => {}
            ListView::Filtered { filter_string, .. } => {
                let mut new_filter_string = std::mem::take(filter_string);
                new_filter_string.pop();
                self.update_filter(new_filter_string);
            }
        }
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
