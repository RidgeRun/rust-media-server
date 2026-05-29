/*
 * Copyright 2026 RidgeRun, LLC (http://www.ridgerun.com)
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 * 1. Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright
 * notice, this list of conditions and the following disclaimer in the
 * documentation and/or other materials provided with the distribution.
 *
 * 3. Neither the name of the copyright holder nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use chrono::Local;

use crate::common::errors::{AppResult, ErrorCode};
use crate::ports::{ILogger, LogLevel};

pub struct FileLogger {
    file: Mutex<File>,
    level: Mutex<LogLevel>,
}

impl FileLogger {
    pub fn new<P: AsRef<Path>>(path: P, append: bool) -> AppResult<Self> {
        let mut options = OpenOptions::new();
        options.create(true).write(true);

        if append {
            options.append(true);
        } else {
            options.truncate(true);
        }

        let file = options.open(path).map_err(|_| ErrorCode::KLoggerError)?;

        Ok(Self {
            file: Mutex::new(file),
            level: Mutex::new(LogLevel::KInfo),
        })
    }

    fn timestamp() -> String {
        Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
    }

    fn should_log(&self, level: LogLevel) -> AppResult<bool> {
        let current_level = *self.level.lock().map_err(|_| ErrorCode::KLoggerError)?;

        Ok(level <= current_level && level != LogLevel::KNone)
    }
}

impl ILogger for FileLogger {
    fn log(&self, message: &str, level: LogLevel) -> AppResult<()> {
        if !self.should_log(level)? {
            return Ok(());
        }

        let mut file = self.file.lock().map_err(|_| ErrorCode::KLoggerError)?;
        writeln!(file, "[{}] [{:?}] {}", Self::timestamp(), level, message)
            .map_err(|_| ErrorCode::KLoggerError)?;

        Ok(())
    }

    fn set_level(&self, level: LogLevel) -> AppResult<()> {
        let mut current_level = self.level.lock().map_err(|_| ErrorCode::KLoggerError)?;
        *current_level = level;
        Ok(())
    }
}
