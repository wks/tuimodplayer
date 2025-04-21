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
    fs::File,
    io::{Cursor, Read, Seek},
};

use openmpt::module::{stream::ModuleStream, Logger, Module};

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::{control::ModuleControl, playlist::ModPath};

#[derive(Debug)]
pub struct ModuleCreationError;

impl std::error::Error for ModuleCreationError {}
impl std::fmt::Display for ModuleCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "libopenmpt openmpt failed to open the module")
    }
}

fn open_module(mut stream: impl ModuleStream) -> Result<Module, ModuleCreationError> {
    Module::create(&mut stream, Logger::None, &[]).map_err(|_| ModuleCreationError)
}

pub fn open_module_from_mod_path(mod_path: &ModPath) -> Result<Module> {
    let file = File::open(&mod_path.file_path)?;

    if mod_path.archive_paths.is_empty() {
        log::info!(
            "Opening root path as module: {}",
            mod_path.file_path.to_string_lossy()
        );
        Ok(open_module(file)?)
    } else {
        log::info!(
            "Opening file in archive: {}",
            mod_path.file_path.to_string_lossy()
        );
        let mut content =
            read_file_from_archive(file, ReadWhatFromArchive::Name(&mod_path.archive_paths[0]))?;

        for archive_path in mod_path.archive_paths[1..].iter() {
            let cursor = Cursor::new(content);
            content = read_file_from_archive(cursor, ReadWhatFromArchive::Name(archive_path))
                .context("Opening inner archive")?;
        }

        if mod_path.is_archived_single {
            let cursor = Cursor::new(content);
            content = read_file_from_archive(cursor, ReadWhatFromArchive::First)
                .context("Opening archived single")?;
        }

        let cursor = Cursor::new(content);
        Ok(open_module(cursor)?)
    }
}

enum ReadWhatFromArchive<'a> {
    Name(&'a str),
    First,
}

fn read_file_from_archive(archive: impl Read + Seek, what: ReadWhatFromArchive) -> Result<Vec<u8>> {
    let mut zip = ZipArchive::new(archive)?;
    let mut zip_file = match what {
        ReadWhatFromArchive::Name(archive_path) => zip.by_name(archive_path)?,
        ReadWhatFromArchive::First => zip.by_index(0)?,
    };
    let zip_file_size = zip_file.size();
    let size = usize::try_from(zip_file_size)
        .map_err(|_| anyhow::anyhow!("File too large: {}", zip_file_size))?;
    let mut content = Vec::with_capacity(size);
    zip_file.read_to_end(&mut content)?;
    Ok(content)
}

pub fn apply_mod_settings(module: &mut Module, control: &ModuleControl) {
    module.ctl_set_play_pitch_factor(control.pitch.output());
    module.ctl_set_play_tempo_factor(control.tempo.output());
    module.set_render_mastergain_millibel(control.gain.output());
    module.set_render_stereo_separation(control.stereo_separation.output());
    module.set_render_interpolation_filter_length(control.filter_taps.output());
    module.set_render_volume_ramping(control.volume_ramping.output());
    module.set_repeat_count(if control.repeat { -1 } else { 0 });
}
