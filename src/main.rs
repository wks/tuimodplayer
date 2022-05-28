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

mod app;
mod logging;
mod module_file;
mod module_source;
mod options;
mod player;
mod playlist;
mod backend;
mod ui;

use clap::Parser;
use options::Options;

fn print_error_and_exit(msg: &str, e: &dyn std::error::Error) -> ! {
    eprintln!("{}: {}", msg, e);
    let mut src = e.source();
    while let Some(e) = src {
        eprintln!("  Cause by: {}", e);
        src = e.source();
    }

    std::process::exit(1);
}

fn main() {
    if let Err(e) = crate::logging::init() {
        print_error_and_exit("Failed to initialize logger", &e);
    }

    let options = Options::parse();
    if let Err(e) = app::run(options) {
        print_error_and_exit("TUIModPlayer exited with an error", e.as_ref());
    }
}
