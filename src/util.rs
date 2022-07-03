use num_traits::{PrimInt, Unsigned, Zero};
use tui::layout::{Constraint, Layout, Rect};

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

use std::fmt::Debug;

/// Compute (a + b) % m
pub fn add_modulo_unsigned<T: PrimInt + Unsigned + Debug>(a: T, b: T, m: T) -> T {
    debug_assert_ne!(m, Zero::zero());
    debug_assert!(a < m);
    debug_assert!(b < m);

    // (a + b) may overflow, but (m - b) may not, given b < m.
    let result = if a >= m - b {
        // a + b >= m.  We need to subtract the result by m.
        a - (m - b) // Equivalent to (a + b - m), but without overflow.
    } else {
        // a + b < m.  Add directly.
        a + b
    };

    debug_assert!(result < m, "result = {:?}, m = {:?}", result, m);
    result
}

/// Compute (a - b) % m
pub fn sub_modulo_unsigned<T: PrimInt + Unsigned + Debug>(a: T, b: T, m: T) -> T {
    debug_assert_ne!(m, Zero::zero());
    debug_assert!(a < m);
    debug_assert!(b < m);

    let result = if a >= b {
        // a >= b.  The result is non-negative.
        a - b
    } else {
        // b > a.  Need to add m to the result.
        // (a + b) may overflow, but (m - b) may not, given b < m.
        a + (m - b) // Equivalent to (a - b + m), but without overflow.
    };

    debug_assert!(result < m);
    result
}

pub trait LayoutSplitN {
    fn split_n<const N: usize>(self, area: Rect, constraints: [Constraint; N]) -> [Rect; N];
}

impl LayoutSplitN for Layout {
    fn split_n<const N: usize>(self, area: Rect, constraints: [Constraint; N]) -> [Rect; N] {
        let results = self.constraints(constraints).split(area);
        assert_eq!(results.len(), N);
        let mut index = 0;
        [(); N].map(|_| {
            let my_index = index;
            index += 1;
            results[my_index]
        })
    }
}

/// Given the length of a list `list_len`,
/// the height of the window `window_len`,
/// and the index of the selected item `selected`,
/// find the ideal offset so that
/// when items in the range `offset..(offset+window_len) are displayed,
/// the selected item is right in the middle of the displayed items.
pub fn center_region(list_len: usize, window_len: usize, selected: usize) -> usize {
    assert!(selected < list_len);
    let result = if list_len <= window_len {
        0
    } else {
        let half_window = window_len / 2;
        if selected <= half_window {
            0
        } else if list_len - selected <= window_len - half_window {
            list_len - window_len
        } else {
            selected - half_window
        }
    };

    // Assert that the selected item is within the window.
    assert!(window_len == 0 || selected >= result);
    assert!(window_len == 0 || selected < result + window_len);

    result
}
