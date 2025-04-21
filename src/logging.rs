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

use std::{collections::VecDeque, sync::Mutex};

use atomic::{Atomic, Ordering};

pub fn init() -> Result<(), log::SetLoggerError> {
    let logger = Box::new(Logger {});
    log::set_boxed_logger(logger).map(|()| log::set_max_level(log::LevelFilter::Trace))
}

pub fn set_stderr_enabled(value: bool) {
    LOGGER_SHARED.enable_stderr.store(value, Ordering::SeqCst)
}

pub fn last_n_records(n: usize) -> Vec<LogRecord> {
    let buffer = LOGGER_SHARED.log_buffer.lock().unwrap();
    buffer.last_n(n)
}

struct LoggerShared {
    enable_stderr: Atomic<bool>,
    log_buffer: Mutex<LogBuffer>,
}

#[derive(Clone)]
pub struct LogRecord {
    pub level: log::Level,
    pub target: String,
    pub message: String,
}

impl std::fmt::Display for LogRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}] {}", self.level, self.target, self.message)
    }
}

struct LogBuffer {
    buffer: VecDeque<LogRecord>,
}

impl LogBuffer {
    const RETAIN: usize = 200;

    pub fn push(&mut self, record: LogRecord) {
        self.buffer.push_back(record);
        while self.buffer.len() > Self::RETAIN {
            self.buffer.pop_front();
        }
    }

    pub fn last_n(&self, n: usize) -> Vec<LogRecord> {
        let len = self.buffer.len();
        self.buffer
            .iter()
            .skip(len.saturating_sub(n))
            .cloned()
            .collect()
    }
}

struct Logger {}

static LOGGER_SHARED: LoggerShared = LoggerShared {
    enable_stderr: Atomic::new(true),
    log_buffer: Mutex::new(LogBuffer {
        buffer: VecDeque::new(),
    }),
};

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let my_record = LogRecord {
                level: record.level(),
                target: record.target().to_string(),
                message: record.args().to_string(),
            };
            let string = my_record.to_string();
            if LOGGER_SHARED.enable_stderr.load(Ordering::SeqCst) {
                eprintln!("{}", string);
            }
            let mut log_buffer = LOGGER_SHARED.log_buffer.lock().unwrap();
            log_buffer.push(my_record);
        }
    }

    fn flush(&self) {}
}
