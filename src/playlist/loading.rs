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

use anyhow::Result;
use lazy_static::lazy_static;

use std::{
    ffi::OsString,
    fs::File,
    io::{BufReader, Cursor, Read, Seek},
    path::Path,
};
use zip::read::ZipFile;

use walkdir::WalkDir;

use crate::playlist::PlayListItem;

use super::{ModPath, PlayList};

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

pub fn extension_is_supported(path: impl AsRef<Path>) -> bool {
    if let Some(ext) = path.as_ref().extension() {
        let ext_lower = ext.to_ascii_lowercase();
        SUPPORTED_EXTENSIONS_OSSTR
            .iter()
            .any(|sup_ext| ext_lower == *sup_ext)
    } else {
        false
    }
}

fn extension_is_archive(path: impl AsRef<Path>) -> bool {
    if let Some(ext) = path.as_ref().extension() {
        let ext_lower = ext.to_ascii_lowercase();
        ext_lower == "zip"
    } else {
        false
    }
}

pub fn load_from_path(playlist: &mut PlayList, root_path: &str) {
    let mut loader = RecursiveModuleLoader::new(|mod_path| {
        playlist.add_item(PlayListItem {
            mod_path,
            metadata: None,
        })
    });

    let time1 = std::time::Instant::now();
    loader.load_from_root_path(Path::new(root_path));
    let duration = time1.elapsed();
    log::debug!("It took {}ms to open {}", duration.as_millis(), root_path);
}

struct RecursiveModuleLoader<F: FnMut(ModPath)> {
    sink: F,
}

impl<F: FnMut(ModPath)> RecursiveModuleLoader<F> {
    pub fn new(sink: F) -> Self {
        Self { sink }
    }

    pub fn load_from_root_path(&mut self, root_path: &Path) {
        if root_path.is_file() {
            self.load_from_file(root_path, root_path);
        } else if root_path.is_dir() {
            self.load_from_dir(root_path, root_path);
        } else {
            log::info!("{:?} is neither a file or a directory", root_path);
        }
    }

    pub fn load_from_file(&mut self, root_path: &Path, path: &Path) {
        debug_assert!(path.is_file()); // Really? What about TOC-TOU?

        log::info!("Path: {:?}", path);

        if extension_is_archive(path) {
            self.load_from_fs_archive_file(root_path, path);
        } else {
            (self.sink)(ModPath {
                root_path: root_path.into(),
                file_path: path.into(),
                archive_paths: vec![],
            });
        }
    }

    pub fn load_from_fs_archive_file(&mut self, root_path: &Path, path: &Path) {
        match buf_open(path) {
            Ok(buf_reader) => {
                let template = ModPath {
                    root_path: root_path.into(),
                    file_path: path.into(),
                    archive_paths: Vec::new(),
                };
                self.load_from_archive(template, buf_reader);
            }
            Err(e) => {
                log::debug!("Skip unopenable archive file: {:?} Error: {}", path, e);
            }
        }
    }

    pub fn load_from_archive(&mut self, template: ModPath, file: impl Read + Seek) {
        match zip::ZipArchive::new(file) {
            Ok(ref mut zip) => {
                for i in 0..zip.len() {
                    match zip.by_index(i) {
                        Ok(zip_file) => {
                            self.load_from_file_in_archive(&template, zip_file);
                        }
                        Err(e) => {
                            log::debug!(
                                "Skip zip entry: {}:{} Error: {}",
                                template.display_full_name(),
                                i,
                                e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                log::debug!(
                    "Skip invalid zip: {} Error: {}",
                    template.display_full_name(),
                    e
                );
            }
        }
    }

    pub fn load_from_file_in_archive(&mut self, template: &ModPath, mut zip_file: ZipFile) {
        let name = zip_file.name().to_string();
        if extension_is_supported(&name) {
            let mut mod_path = template.clone();
            mod_path.archive_paths.push(name);
            (self.sink)(mod_path);
        } else if extension_is_archive(&name) {
            let mut sub_template = template.clone();
            sub_template.archive_paths.push(name.clone());
            let mut content = Vec::new();
            match zip_file.read_to_end(&mut content) {
                Ok(_) => {
                    let cursor = Cursor::new(content);
                    self.load_from_archive(sub_template, cursor);
                }
                Err(e) => {
                    log::debug!(
                        "Cannot open inner archive {}:{} Error: {}",
                        template.display_full_name(),
                        name,
                        e
                    );
                }
            }
        } else {
            log::debug!(
                "Unrecognised zip content: {}:{}",
                template.display_full_name(),
                name
            );
        }
    }

    pub fn load_from_dir(&mut self, root_path: &Path, dir_path: &Path) {
        debug_assert!(dir_path.is_dir()); // Really? What about TOC-TOU?

        WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|r| r.ok())
            .for_each(|de| {
                let file_path = de.path();
                if extension_is_supported(file_path) {
                    (self.sink)(ModPath {
                        root_path: root_path.into(),
                        file_path: file_path.into(),
                        archive_paths: vec![],
                    })
                } else if extension_is_archive(file_path) {
                    self.load_from_fs_archive_file(root_path, file_path)
                }
            })
    }
}

fn buf_open(path: &Path) -> Result<BufReader<File>> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    Ok(buf_reader)
}
