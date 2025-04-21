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

use std::{ffi::OsString, path::Path};

#[derive(Clone)]
pub struct ModPath {
    pub root_path: OsString,
    pub file_path: OsString,
    pub archive_paths: Vec<String>,
    pub is_archived_single: bool,
}

impl ModPath {
    pub fn display_name(&self) -> String {
        if self.archive_paths.is_empty() {
            let file_path = Path::new(&self.file_path);
            file_path
                .file_name()
                .unwrap_or(self.file_path.as_os_str())
                .to_string_lossy()
                .into()
        } else {
            self.archive_paths.last().unwrap().into()
        }
    }

    pub fn display_full_name(&self) -> String {
        let file_path = self.file_path.to_string_lossy();
        if self.archive_paths.is_empty() {
            file_path.to_string()
        } else {
            format!("{}:{}", file_path, self.archive_paths.join(":"))
        }
    }
}

pub struct ModMetadata {
    pub title: String,
}

pub struct PlayListItem {
    pub mod_path: ModPath,
    pub metadata: Option<ModMetadata>,
}
