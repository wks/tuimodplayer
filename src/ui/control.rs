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

use crate::app::AppState;

use crossterm::event::{self, KeyModifiers};

use event::{Event, KeyCode, KeyEvent};

pub enum HandleKeyResult {
    Nothing,
    Redraw,
    Quit,
}

pub fn handle_key_event(ev: &Event, app_state: &mut AppState) -> HandleKeyResult {
    #[allow(clippy::single_match)] // Will add more event handling in the future
    #[allow(clippy::collapsible_match)]
    match ev {
        Event::Key(KeyEvent {
            code, modifiers, ..
        }) => match code {
            KeyCode::Char('l') if modifiers.contains(KeyModifiers::CONTROL) => {
                return HandleKeyResult::Redraw;
            }
            KeyCode::Char('q') => {
                return HandleKeyResult::Quit;
            }
            KeyCode::Char('m') => {
                app_state.next();
            }
            KeyCode::Char('n') => {
                app_state.prev();
            }
            KeyCode::Char('M') => {
                app_state.next10();
            }
            KeyCode::Char('N') => {
                app_state.prev10();
            }
            KeyCode::Char('u') => {
                app_state.tempo_down();
            }
            KeyCode::Char('i') => {
                app_state.tempo_up();
            }
            KeyCode::Char('o') => {
                app_state.pitch_down();
            }
            KeyCode::Char('p') => {
                app_state.pitch_up();
            }
            KeyCode::Char('3') => {
                app_state.gain_down();
            }
            KeyCode::Char('4') => {
                app_state.gain_up();
            }
            KeyCode::Char('5') => {
                app_state.stereo_separation_down();
            }
            KeyCode::Char('6') => {
                app_state.stereo_separation_up();
            }
            KeyCode::Char('7') => {
                app_state.filter_taps_down();
            }
            KeyCode::Char('8') => {
                app_state.filter_taps_up();
            }
            KeyCode::Char('9') => {
                app_state.volume_ramping_down();
            }
            KeyCode::Char('0') => {
                app_state.volume_ramping_up();
            }
            KeyCode::Char('r') => {
                app_state.toggle_repeat();
            }
            KeyCode::Char(' ') => {
                app_state.pause_resume();
            }
            _ => {}
        },
        _ => {}
    }

    HandleKeyResult::Nothing
}
