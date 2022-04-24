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

use std::path::Path;

pub struct ModPath {
    pub root_path: String,
    pub archive_paths: Vec<String>,
}

pub struct ModMetadata {
    pub title: String,
}

pub struct PlayListItem {
    pub mod_path: ModPath,
    pub metadata: Option<ModMetadata>,
}

pub fn load_from_path(root_path: &str) -> Vec<PlayListItem> {
    let path = Path::new(&root_path);

    let mut items = vec![];

    let mut add_item = |mod_path: ModPath| {
        items.push(PlayListItem {
            mod_path,
            metadata: None,
        });
    };

    if path.is_file() {
        load_from_file(root_path, path, &mut add_item);
    } else if path.is_dir() {
        todo!("Load from dir");
    } else {
        log::info!("{} is neither a file or a directory", root_path);
    }

    items
}

fn load_from_file<F: FnMut(ModPath)>(root_path: &str, path: &Path, f: &mut F) {
    debug_assert!(path.is_file()); // Really? What about TOC-TOU?

    if path.ends_with(".zip") {
        todo!("Load from zip");
    } else {
        f(ModPath {
            root_path: root_path.to_string(),
            archive_paths: vec![],
        });
    }
}
