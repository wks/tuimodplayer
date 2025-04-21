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

mod control;
mod display;

use std::{io::stdout, panic::PanicHookInfo, sync::OnceLock, time::Duration};

use crate::app::AppState;

use crossterm::{event, execute, terminal};

use anyhow::Result;

use self::{
    control::{handle_key_event, HandleKeyResult},
    display::render_ui,
};

type BoxedHook = Box<dyn Fn(&PanicHookInfo) + Sync + Send>;

struct PanicHookRegistration {
    old_hook: BoxedHook,
}

static PANIC_HOOK_REGISTRATION: OnceLock<PanicHookRegistration> = OnceLock::new();

fn ui_panic_hook(panic_info: &PanicHookInfo<'_>) {
    execute!(stdout(), terminal::LeaveAlternateScreen).unwrap_or_else(|e| {
        // Cannot handle error while handling panic.  Printing is the best effort.
        eprintln!("Failed to leave alternative screen: {}", e);
    });
    crate::logging::set_stderr_enabled(true);
    terminal::disable_raw_mode().unwrap_or_else(|e| {
        // Cannot handle error while handling panic.  Printing is the best effort.
        eprintln!("Failed to disable raw mode: {}", e);
    });
    let old_hook = &PANIC_HOOK_REGISTRATION
        .get()
        .expect("ui_panic_hook called but PANIC_HOOK_REGISTRATION is not initialized.")
        .old_hook;
    old_hook(panic_info);
}

pub fn run_ui(app_state: &mut AppState) -> Result<()> {
    PANIC_HOOK_REGISTRATION.get_or_init(|| {
        let old_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(ui_panic_hook));
        PanicHookRegistration { old_hook }
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
