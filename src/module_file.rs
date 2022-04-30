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

use std::fs::File;

use openmpt::module::{stream::ModuleStream, Logger, Module};

use anyhow::{anyhow, Result};

use crate::playlist::ModPath;

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

pub fn open_module_file(file_path: String) -> Result<Module> {
    let file = File::open(&file_path)?;

    let module = if file_path.ends_with(".zip") {
        let mut zip = zip::ZipArchive::new(file)?;

        if zip.len() != 1 {
            return Err(anyhow!("This zip archive has more than one file inside."));
        }

        let inner_file = zip.by_index(0)?;
        log::info!("Using {} from zip file {}", inner_file.name(), file_path);
        open_module(inner_file)
    } else {
        log::info!("Using file {} directly", file_path);
        open_module(file)
    }?;

    Ok(module)
}

pub fn open_module_from_mod_path(mod_path: &ModPath) -> Result<Module> {
    let file = File::open(&mod_path.root_path)?;

    if mod_path.archive_paths.is_empty() {
        log::info!(
            "Opening root path as module: {}",
            mod_path.root_path.to_string_lossy()
        );
        Ok(open_module(file)?)
    } else {
        todo!("Open from nested archives")
    }
}
