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

use ratatui::style::{Color, Modifier, Style};

pub struct ColorScheme {
    pub normal: Style,
    pub key: Style,
    pub block_title: Style,
    pub list_highlight: Style,
    pub log_error: Style,
    pub log_warn: Style,
    pub log_info: Style,
    pub log_debug: Style,
    pub log_trace: Style,
    pub log_target: Style,
    pub log_message: Style,
}

impl Default for ColorScheme {
    fn default() -> Self {
        let base = Style::default().fg(Color::Gray).bg(Color::Black);
        let log_base = base.add_modifier(Modifier::BOLD);
        Self {
            normal: base,
            key: base.fg(Color::White).add_modifier(Modifier::BOLD),
            block_title: base.add_modifier(Modifier::BOLD),
            list_highlight: Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
            log_error: log_base.fg(Color::Red),
            log_warn: log_base.fg(Color::Magenta),
            log_info: log_base.fg(Color::Green),
            log_debug: log_base.fg(Color::Blue),
            log_trace: log_base.fg(Color::Yellow),
            log_target: log_base.fg(Color::DarkGray),
            log_message: base,
        }
    }
}
