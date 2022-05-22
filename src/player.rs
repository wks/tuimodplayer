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

use atomic::Atomic;

#[derive(Default)]
pub struct PlayState {
    pub order: Atomic<usize>,
    pub pattern: Atomic<usize>,
    pub row: Atomic<usize>,
    pub n_rows: Atomic<usize>,
    pub speed: Atomic<usize>,
    pub tempo: Atomic<usize>,
}
