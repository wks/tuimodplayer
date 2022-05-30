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

use std::num::IntErrorKind;

use clap::Parser;

/// The default sample rate.
///
/// libopenmpt recommends 48000 because
/// "practically all audio equipment and file formats use 48000Hz nowadays".
pub const DEFAULT_SAMPLE_RATE: usize = 48000;

/// Minimum sample rate supported by libopenmpt.
pub const MIN_SAMPLE_RATE: usize = 8000;

/// Maximum sample rate supported by libopenmpt.
pub const MAX_SAMPLE_RATE: usize = 192000;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
pub struct Options {
    /// Paths to individual mods, archives or directories.
    /// For archives and directories, it will search for all mod files inside.
    #[clap(name = "PATH")]
    pub paths: Vec<String>,
    #[clap(
        long,
        default_value_t = DEFAULT_SAMPLE_RATE,
        validator = parse_sample_rate
    )]
    pub sample_rate: usize,
}

enum RangeParseError {
    TooLow,
    TooHigh,
    Invalid,
}

fn usize_range_parse(v: &str, low: usize, high: usize) -> Result<usize, RangeParseError> {
    let num = v.parse::<usize>().map_err(|e| match e.kind() {
        IntErrorKind::Empty => RangeParseError::Invalid,
        IntErrorKind::InvalidDigit => RangeParseError::Invalid,
        IntErrorKind::PosOverflow => RangeParseError::TooHigh,
        IntErrorKind::NegOverflow => RangeParseError::TooLow,
        IntErrorKind::Zero => unreachable!("Zero is still within the range of usize"),
        _ => RangeParseError::Invalid,
    })?;
    if num < low {
        Err(RangeParseError::TooLow)
    } else if num > high {
        Err(RangeParseError::TooHigh)
    } else {
        Ok(num)
    }
}

fn parse_sample_rate(v: &str) -> Result<usize, String> {
    usize_range_parse(v, MIN_SAMPLE_RATE, MAX_SAMPLE_RATE).map_err(|e| match e {
        RangeParseError::Invalid => format!(
            "Expected integer within {}-{}",
            MIN_SAMPLE_RATE, MAX_SAMPLE_RATE
        ),
        RangeParseError::TooLow | RangeParseError::TooHigh => format!(
            "Out of range.  Supported sample rate range: {}-{}",
            MIN_SAMPLE_RATE, MAX_SAMPLE_RATE
        ),
    })
}
