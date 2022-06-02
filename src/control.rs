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

use num_traits::Num;

#[derive(Clone)]
pub struct ModuleControl {
    pub tempo: ControlField,
    pub pitch: ControlField,
}

impl Default for ModuleControl {
    fn default() -> Self {
        Self {
            tempo: ControlField::new(&controls::TEMPO),
            pitch: ControlField::new(&controls::PITCH),
        }
    }
}

mod controls {
    use super::{ControlScale, ControlSpec};

    pub const TEMPO: ControlSpec<i32> = ControlSpec::<i32> {
        low: -48,
        high: 48,
        default: 0,
        step: 1,
        scale: ControlScale::Logarithmic {
            base: 2.0,
            denominator: 24.0,
        },
    };

    pub const PITCH: ControlSpec<i32> = ControlSpec::<i32> {
        low: -48,
        high: 48,
        default: 0,
        step: 1,
        scale: ControlScale::Logarithmic {
            base: 2.0,
            denominator: 24.0,
        },
    };
}

#[derive(Clone)]
pub struct ControlField {
    value: i32,
    spec: &'static ControlSpec<i32>,
}

impl ControlField {
    pub fn new(spec: &'static ControlSpec<i32>) -> Self {
        Self {
            value: spec.default,
            spec,
        }
    }

    pub fn inc(&mut self) {
        self.value = (self.value + self.spec.step).min(self.spec.high);
    }

    pub fn dec(&mut self) {
        self.value = (self.value - self.spec.step).max(self.spec.low);
    }

    pub fn output(&self) -> f64 {
        let value_f64 = self.value as f64;
        match self.spec.scale {
            ControlScale::Linear { factor, offset } => (value_f64) * factor + offset,
            ControlScale::Logarithmic { base, denominator } => base.powf(value_f64 / denominator),
        }
    }
}

pub struct ControlSpec<T: Num> {
    low: T,
    high: T,
    default: T,
    step: T,
    scale: ControlScale,
}

pub enum ControlScale {
    /// Linear scale.  `y = x * factor + offset`
    Linear { factor: f64, offset: f64 },
    /// Logrithmic scale.  `y = base ^ (x / denominator)`
    Logarithmic { base: f64, denominator: f64 },
}
