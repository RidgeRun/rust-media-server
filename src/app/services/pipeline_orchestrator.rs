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

use crate::common::errors::{AppResult, ErrorCode};
use crate::common::logging::log_message;
use crate::common::pipeline_config::SourcePipelineConfig;
use crate::ports::{IPipeline, LogLevel, SharedLogger};

pub struct PipelineOrchestrator {
    pipeline_manager: Box<dyn IPipeline>,
    logger: SharedLogger,
}

impl PipelineOrchestrator {
    pub fn new(pipeline_manager: Box<dyn IPipeline>, logger: SharedLogger) -> Self {
        Self {
            pipeline_manager,
            logger,
        }
    }

    pub fn create_all(&mut self, configs: &[SourcePipelineConfig]) -> AppResult<()> {
        for source in configs {
            let _ = log_message(
                &self.logger,
                &format!(
                    "[debug] creating source pipeline: {}\n[debug] description: {}",
                    source.name, source.description
                ),
                LogLevel::KDebug,
            );
            self.pipeline_manager
                .create_pipeline(&source.name, &source.description)?;

            for feature in &source.features {
                let _ = log_message(
                    &self.logger,
                    &format!(
                        "[debug] creating feature pipeline: {}\n[debug] description: {}",
                        feature.name, feature.description
                    ),
                    LogLevel::KDebug,
                );
                self.pipeline_manager
                    .create_pipeline(&feature.name, &feature.description)?;
            }
        }

        Ok(())
    }

    pub fn play_all(&mut self, configs: &[SourcePipelineConfig]) -> AppResult<()> {
        for source in configs {
            let _ = log_message(
                &self.logger,
                &format!(
                    "[debug] playing source pipeline: {}\n[debug] description: {}",
                    source.name, source.description
                ),
                LogLevel::KDebug,
            );
            self.pipeline_manager.play_pipeline(&source.name)?;

            for feature in &source.features {
                let _ = log_message(
                    &self.logger,
                    &format!(
                        "[debug] playing feature pipeline: {}\n[debug] description: {}",
                        feature.name, feature.description
                    ),
                    LogLevel::KDebug,
                );
                self.pipeline_manager.play_pipeline(&feature.name)?;
            }
        }

        Ok(())
    }

    pub fn stop_all(&mut self, configs: &[SourcePipelineConfig]) -> AppResult<()> {
        let mut had_error = false;

        for source in configs {
            for feature in &source.features {
                if self.pipeline_manager.stop_pipeline(&feature.name).is_err() {
                    had_error = true;
                }
            }

            if self.pipeline_manager.stop_pipeline(&source.name).is_err() {
                had_error = true;
            }
        }

        if had_error {
            Err(ErrorCode::KGstCError)
        } else {
            Ok(())
        }
    }

    pub fn get_state(&self, pipeline_name: &str) -> AppResult<String> {
        self.pipeline_manager.get_state(pipeline_name)
    }
}
