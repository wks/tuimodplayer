use num_traits::{PrimInt, Unsigned, Zero};

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
