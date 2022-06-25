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

mod control;
mod display;

use std::{io::stdout, panic::PanicInfo, time::Duration};

use crate::app::AppState;

use crossterm::{event, execute, terminal};

use anyhow::Result;

use self::{
    control::{handle_key_event, HandleKeyResult},
    display::render_ui,
};

static mut OLD_HOOK: Option<Box<dyn Fn(&PanicInfo) + Sync + Send>> = None;
static REGISTER_PANIC_HOOK: std::sync::Once = std::sync::Once::new();

fn ui_panic_hook(panic_info: &PanicInfo<'_>) {
    execute!(stdout(), terminal::LeaveAlternateScreen).unwrap_or_else(|e| {
        // Cannot handle error while handling panic.  Printing is the best effort.
        eprintln!("Failed to leave alternative screen: {}", e);
    });
    crate::logging::set_stderr_enabled(true);
    terminal::disable_raw_mode().unwrap_or_else(|e| {
        // Cannot handle error while handling panic.  Printing is the best effort.
        eprintln!("Failed to disable raw mode: {}", e);
    });
    let old_hook = unsafe { OLD_HOOK.as_ref().unwrap() };
    old_hook(panic_info);
}

pub fn run_ui(app_state: &mut AppState) -> Result<()> {
    REGISTER_PANIC_HOOK.call_once(|| {
        unsafe {
            OLD_HOOK = Some(std::panic::take_hook());
        }
        std::panic::set_hook(Box::new(ui_panic_hook));
    });

    terminal::enable_raw_mode()?;

    crate::logging::set_stderr_enabled(false);
    execute!(stdout(), terminal::EnterAlternateScreen)?;

    let backend = tui::backend::CrosstermBackend::new(stdout());
    let mut term = tui::Terminal::new(backend)?;

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
            let area = frame.size();
            render_ui(frame, area, app_state);
        })?;
    }

    execute!(stdout(), terminal::LeaveAlternateScreen)?;
    crate::logging::set_stderr_enabled(true);

    terminal::disable_raw_mode()?;

    Ok(())
}
