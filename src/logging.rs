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

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use atomic::{Atomic, Ordering};
use lazy_static::lazy_static;

pub fn init() -> Result<(), log::SetLoggerError> {
    let logger = Box::new(Logger {
        shared: LOGGER_SHARED.clone(),
    });
    log::set_boxed_logger(logger).map(|()| log::set_max_level(log::LevelFilter::Trace))
}

pub fn set_stderr_enabled(value: bool) {
    LOGGER_SHARED.enable_stderr.store(value, Ordering::SeqCst)
}

pub fn last_n_records(n: usize) -> Vec<String> {
    let buffer = LOGGER_SHARED.log_buffer.lock().unwrap();
    buffer.last_n(n)
}

struct LoggerShared {
    enable_stderr: Atomic<bool>,
    log_buffer: Mutex<LogBuffer>,
}

struct LogBuffer {
    buffer: VecDeque<String>,
}

impl LogBuffer {
    const RETAIN: usize = 200;

    pub fn push(&mut self, string: String) {
        self.buffer.push_back(string);
        while self.buffer.len() > Self::RETAIN {
            self.buffer.pop_front();
        }
    }

    pub fn last_n(&self, n: usize) -> Vec<String> {
        let len = self.buffer.len();
        self.buffer
            .iter()
            .skip(len.saturating_sub(n))
            .cloned()
            .collect()
    }
}

struct Logger {
    shared: Arc<LoggerShared>,
}

lazy_static! {
    static ref LOGGER_SHARED: Arc<LoggerShared> = Arc::new(LoggerShared {
        enable_stderr: Atomic::new(true),
        log_buffer: Mutex::new(LogBuffer {
            buffer: Default::default(),
        }),
    });
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let string = format!("{} - {}", record.level(), record.args());
            if self.shared.enable_stderr.load(Ordering::SeqCst) {
                eprintln!("{}", string);
            }
            let mut log_buffer = self.shared.log_buffer.lock().unwrap();
            log_buffer.push(string);
        }
    }

    fn flush(&self) {}
}
