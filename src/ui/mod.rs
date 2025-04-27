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

pub mod color_scheme;
mod control;
mod display;

use std::time::Duration;

use crate::app::AppState;

use crossterm::event;

use anyhow::Result;

use self::{
    control::{HandleKeyResult, handle_key_event},
    display::render_ui,
};

pub fn run_ui(app_state: &mut AppState) -> Result<()> {
    let mut term = ratatui::try_init()?;
    crate::logging::set_stderr_enabled(false);

    'event_loop: loop {
        let mut redraw = false;

        if event::poll(Duration::from_millis(100))? {
            let ev = event::read()?;
            let key_event_result = handle_key_event(&ev, app_state);
            match key_event_result {
                HandleKeyResult::Nothing => {}
                HandleKeyResult::Redraw => {
                    redraw = true;
                }
                HandleKeyResult::Quit => {
                    break 'event_loop;
                }
            }
        }

        app_state.handle_backend_events();

        if std::mem::take(&mut redraw) {
            term.clear()?;
        }

        term.draw(|frame| {
            render_ui(frame, app_state);
        })?;
    }

    crate::logging::set_stderr_enabled(true);

    ratatui::try_restore()?;

    Ok(())
}
