use num_traits::{PrimInt, Unsigned, Zero};
use tui::{
    layout::{Constraint, Layout, Rect},
    text::{Span, Spans, Text},
};

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

use std::{borrow::Cow, fmt::Debug};

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

/// Return the width of a string when printed on the screen.
/// Currently we just use the number of characters
/// because mod files may not (really?) contain full-width characters,
/// such as Chinese characters, which occupy the width of two letters.
pub fn screen_width(s: &str) -> usize {
    s.chars().count()
}

/// Wrap lines of a `Text` to a fixed width.
/// Oblivious of "word".  Behave more like consoles.
#[allow(unused)]
pub fn force_wrap_text<'a>(text: &Text<'a>, width: usize) -> Text<'a> {
    Text {
        lines: text
            .lines
            .iter()
            .flat_map(|s| force_wrap_spans(s, width))
            .collect(),
    }
}

pub fn force_wrap_spans<'b>(spans: &Spans<'_>, width: usize) -> Vec<Spans<'b>> {
    let mut lines: Vec<Spans> = vec![];
    let mut current_line = vec![];
    let mut line_rem_len = width;
    for span in spans.0.iter() {
        let content_len = span.content.len();
        let mut content_cursor = 0;
        while content_len - content_cursor > line_rem_len {
            let portion_content =
                span.content[content_cursor..content_cursor + line_rem_len].to_string();
            content_cursor += line_rem_len;

            let small_span = Span {
                content: Cow::Owned(portion_content),
                style: span.style,
            };
            current_line.push(small_span);
            lines.push(Spans(current_line));

            current_line = vec![];
            line_rem_len = width;
        }

        assert!(content_len - content_cursor <= line_rem_len);

        if content_len - content_cursor > 0 {
            let portion_content = span.content[content_cursor..].to_string();
            let small_span = Span {
                content: Cow::Owned(portion_content),
                style: span.style,
            };
            current_line.push(small_span);
        }
    }
    if !current_line.is_empty() {
        lines.push(Spans(current_line))
    }
    lines
}

/// I just want to use the unstable feature now.
pub trait IsSomeAnd {
    type T;
    fn is_some_and2(&self, f: impl FnOnce(&Self::T) -> bool) -> bool;
}

impl<T> IsSomeAnd for Option<T> {
    type T = T;
    fn is_some_and2(&self, f: impl FnOnce(&T) -> bool) -> bool {
        match self {
            None => false,
            Some(x) => f(x),
        }
    }
}
