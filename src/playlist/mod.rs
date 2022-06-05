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
use std::{
    borrow::Cow,
    ffi::OsString,
    fs::File,
    path::Path,
    sync::{Arc, Mutex},
};
use zip::read::ZipFile;

use walkdir::WalkDir;

use crate::{
    backend::ModuleProvider,
    module_file::open_module_from_mod_path,
    util::{add_modulo_unsigned, sub_modulo_unsigned},
};

pub struct ModPath {
    pub root_path: OsString,
    pub file_path: OsString,
    pub archive_paths: Vec<String>,
}

impl ModPath {
    pub fn display_name(&self) -> Cow<'_, str> {
        let file_path = Path::new(&self.file_path);
        file_path
            .file_name()
            .unwrap_or(self.file_path.as_os_str())
            .to_string_lossy()
    }
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

pub fn extension_is_supported(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_ascii_lowercase();
        SUPPORTED_EXTENSIONS_OSSTR
            .iter()
            .any(|sup_ext| ext_lower == *sup_ext)
    } else {
        false
    }
}

pub fn extension_is_archive(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_ascii_lowercase();
        ext_lower == "zip"
    } else {
        false
    }
}

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

    pub fn load_from_path(&mut self, root_path: &str) {
        load_from_path(root_path, &mut self.items);
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
        load_from_dir(root_path, path, &mut add_item);
    } else {
        log::info!("{} is neither a file or a directory", root_path);
    }
}

fn load_from_file<F: FnMut(ModPath)>(root_path: &str, path: &Path, f: &mut F) {
    debug_assert!(path.is_file()); // Really? What about TOC-TOU?

    log::info!("Path: {:?}", path);
    let is_archive = if let Some(extension) = path.extension() {
        extension.to_ascii_lowercase() == "zip"
    } else {
        false
    };

    if is_archive {
        load_from_archive(root_path, path, f);
    } else {
        f(ModPath {
            root_path: root_path.into(),
            file_path: path.into(),
            archive_paths: vec![],
        });
    }
}

fn load_from_dir<F: FnMut(ModPath)>(root_path: &str, path: &Path, f: &mut F) {
    debug_assert!(path.is_dir()); // Really? What about TOC-TOU?

    WalkDir::new(path)
        .into_iter()
        .filter_map(|r| r.ok())
        .for_each(|de| {
            let file_path = de.path();
            if extension_is_supported(file_path) {
                f(ModPath {
                    root_path: root_path.into(),
                    file_path: file_path.into(),
                    archive_paths: vec![],
                })
            }
        })
}

fn for_each_file_in_archive<F: FnMut(ZipFile<'_>)>(path: &Path, mut f: F) {
    match File::open(path) {
        Ok(file) => match zip::ZipArchive::new(file) {
            Ok(ref mut zip) => {
                for i in 0..zip.len() {
                    match zip.by_index(i) {
                        Ok(zip_file) => f(zip_file),
                        Err(e) => {
                            log::debug!("Skip invalid zip: {:?} Error: {}", path, e);
                        }
                    }
                }
            }
            Err(e) => {
                log::debug!("Skip invalid zip: {:?} Error: {}", path, e);
            }
        },
        Err(e) => {
            log::debug!("Skip unopenable file: {:?} Error: {}", path, e);
        }
    }
}

fn load_from_archive<F: FnMut(ModPath)>(_root_path: &str, path: &Path, _f: &mut F) {
    debug_assert!(extension_is_archive(path), "{:?} is not archive", path);

    for_each_file_in_archive(path, |zip_file| {
        log::info!("I see file: {}", zip_file.name());
    });
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
