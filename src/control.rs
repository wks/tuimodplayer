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

use num_traits::{FromPrimitive, Num};

#[derive(Clone)]
pub struct ModuleControl {
    pub tempo: ControlField<f64>,
    pub pitch: ControlField<f64>,
    pub gain: ControlField<i32>,
    pub stereo_separation: ControlField<i32>,
    pub filter_taps: ControlField<i32>,
    pub volume_ramping: ControlField<i32>,
    pub repeat: bool,
}

impl Default for ModuleControl {
    fn default() -> Self {
        Self {
            tempo: ControlField::new(&controls::TEMPO),
            pitch: ControlField::new(&controls::PITCH),
            gain: ControlField::new(&controls::GAIN),
            stereo_separation: ControlField::new(&controls::STEREO_SEPARATION),
            filter_taps: ControlField::new(&controls::FILTER_TAPS),
            volume_ramping: ControlField::new(&controls::VOLUME_RAMPING),
            repeat: false,
        }
    }
}

mod controls {
    use super::{ControlScale, ControlSpec};

    pub const TEMPO: ControlSpec<f64> = ControlSpec {
        low: -48,
        high: 48,
        default: 0,
        step: 1,
        scale: ControlScale::Logarithmic {
            base: 2.0,
            denominator: 24.0,
        },
    };

    pub const PITCH: ControlSpec<f64> = ControlSpec {
        low: -48,
        high: 48,
        default: 0,
        step: 1,
        scale: ControlScale::Logarithmic {
            base: 2.0,
            denominator: 24.0,
        },
    };

    pub const GAIN: ControlSpec<i32> = ControlSpec {
        low: i32::MIN,
        high: i32::MAX,
        default: 0,
        step: 1,
        scale: ControlScale::Linear {
            factor: 100,
            offset: 0,
        },
    };

    pub const STEREO_SEPARATION: ControlSpec<i32> = ControlSpec {
        low: 0,
        high: i32::MAX,
        default: 100,
        step: 5,
        scale: ControlScale::Linear {
            factor: 1,
            offset: 0,
        },
    };

    pub const FILTER_TAPS: ControlSpec<i32> = ControlSpec {
        low: 0,
        high: 3,
        default: 3,
        step: 1,
        scale: ControlScale::Logarithmic {
            base: 2.0, // For powers of two, the pow operation is still precise.
            denominator: 1.0,
        },
    };

    pub const VOLUME_RAMPING: ControlSpec<i32> = ControlSpec {
        low: -1,
        high: 10,
        default: -1,
        step: 1,
        scale: ControlScale::Linear {
            factor: 1,
            offset: 0,
        },
    };
}

#[derive(Clone)]
pub struct ControlField<T: Num + FromPrimitive + Copy + 'static> {
    value: i32,
    spec: &'static ControlSpec<T>,
}

impl<T: Num + Copy + FromPrimitive> ControlField<T> {
    pub fn new(spec: &'static ControlSpec<T>) -> Self {
        Self {
            value: spec.default,
            spec,
        }
    }

    pub fn inc(&mut self) {
        self.value = self
            .value
            .saturating_add(self.spec.step)
            .min(self.spec.high);
    }

    pub fn dec(&mut self) {
        self.value = self.value.saturating_sub(self.spec.step).max(self.spec.low);
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn output(&self) -> T {
        match self.spec.scale {
            ControlScale::Linear { factor, offset } => {
                let value_t = T::from_i32(self.value)
                    .unwrap_or_else(|| panic!("Cannot convert {} to T", self.value));
                value_t * factor + offset
            }
            ControlScale::Logarithmic { base, denominator } => {
                let value_f64 = self.value as f64;
                let result_f64 = base.powf(value_f64 / denominator);
                T::from_f64(result_f64)
                    .unwrap_or_else(|| panic!("Cannot convert {} to T", result_f64))
            }
        }
    }
}

pub struct ControlSpec<T: Num> {
    low: i32,
    high: i32,
    default: i32,
    step: i32,
    scale: ControlScale<T>,
}

pub enum ControlScale<T> {
    /// Linear scale.  `y = x * factor + offset`
    Linear { factor: T, offset: T },
    /// Logrithmic scale.  `y = base ^ (x / denominator)`
    Logarithmic { base: f64, denominator: f64 },
}
