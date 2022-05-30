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

use lazy_static::lazy_static;
use openmpt::module::Module;
use std::{ffi::OsString, path::Path, sync::Arc};

use walkdir::WalkDir;

use crate::{backend::ModuleProvider, module_file::open_module_from_mod_path};

pub struct ModPath {
    pub root_path: OsString,
    pub archive_paths: Vec<String>,
}

pub struct ModMetadata {
    pub title: String,
}

pub struct PlayListItem {
    pub mod_path: ModPath,
    pub metadata: Option<ModMetadata>,
}

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mptm", "mod", "s3m", "xm", "it", "669", "amf", "ams", "c67", "dbm", "digi", "dmf", "dsm",
    "dsym", "dtm", "far", "fmt", "imf", "ice", "j2b", "m15", "mdl", "med", "mms", "mt2", "mtm",
    "mus", "nst", "okt", "plm", "psm", "pt36", "ptm", "sfx", "sfx2", "st26", "stk", "stm", "stx",
    "stp", "symmod", "ult", "wow", "gdm", "mo3", "oxm", "umx", "xpk", "ppm", "mmcmp",
];

lazy_static! {
    static ref SUPPORTED_EXTENSIONS_OSSTR: Vec<OsString> = {
        SUPPORTED_EXTENSIONS
            .iter()
            .map(|s| s.into())
            .collect::<Vec<_>>()
    };
}

pub struct PlayList {
    pub items: Vec<PlayListItem>,
    pub now_playing: Option<usize>,
    pub next_index: Option<usize>,
}

impl PlayList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            now_playing: None,
            next_index: None,
        }
    }

    pub fn load_from_path(&mut self, root_path: &str) {
        load_from_path(root_path, &mut self.items);
    }
}

pub fn load_from_path(root_path: &str, items: &mut Vec<PlayListItem>) {
    let path = Path::new(&root_path);

    let mut add_item = |mod_path: ModPath| {
        items.push(PlayListItem {
            mod_path,
            metadata: None,
        });
    };

    if path.is_file() {
        load_from_file(root_path, path, &mut add_item);
    } else if path.is_dir() {
        load_from_dir(path, &mut add_item);
    } else {
        log::info!("{} is neither a file or a directory", root_path);
    }
}

fn load_from_file<F: FnMut(ModPath)>(root_path: &str, path: &Path, f: &mut F) {
    debug_assert!(path.is_file()); // Really? What about TOC-TOU?

    if path.ends_with(".zip") {
        todo!("Load from zip");
    } else {
        f(ModPath {
            root_path: root_path.into(),
            archive_paths: vec![],
        });
    }
}

fn load_from_dir<F: FnMut(ModPath)>(path: &Path, f: &mut F) {
    debug_assert!(path.is_dir()); // Really? What about TOC-TOU?

    WalkDir::new(path)
        .into_iter()
        .filter_map(|r| r.ok())
        .for_each(|de| {
            let path2 = de.path();
            if let Some(ext) = path2.extension() {
                let ext_lower = ext.to_ascii_lowercase();
                if SUPPORTED_EXTENSIONS_OSSTR
                    .iter()
                    .any(|sup_ext| ext_lower == *sup_ext)
                {
                    f(ModPath {
                        root_path: path2.into(),
                        archive_paths: vec![],
                    })
                }
            }
        })
}

pub struct PlayListModuleProvider {
    playlist: Arc<PlayList>,
    cursor: usize,
}

impl PlayListModuleProvider {
    pub fn new(playlist: Arc<PlayList>) -> Self {
        Self { playlist, cursor: 0 }
    }
}

impl ModuleProvider for PlayListModuleProvider {
    fn next_module(&mut self) -> Option<Module> {
        if self.cursor < self.playlist.items.len() {
            let item = &self.playlist.items[self.cursor];
            self.cursor += 1;
            match open_module_from_mod_path(&item.mod_path) {
                Ok(module) => Some(module),
                Err(e) => {
                    log::error!(
                        "Error loading module {:?}: {}",
                        item.mod_path.root_path.to_string_lossy(),
                        e
                    );
                    None
                }
            }
        } else {
            log::info!("No more mods to play!");
            None
        }
    }
}
