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

use std::fs;
use std::path::PathBuf;

use crate::app::factory::{MediaServerFactory, MediaServerStartupConfig};
use crate::app::services::PipelineOrchestrator;
use crate::common::errors::{AppResult, ErrorCode};
use crate::common::pipeline_config::SourcePipelineConfig;

pub struct MediaServer {
    pipelines: Vec<SourcePipelineConfig>,
    orchestrator: PipelineOrchestrator,
}

impl MediaServer {
    pub fn new(
        config_path: PathBuf,
        log_output_path: PathBuf,
        log_level: i32,
        log_append: bool,
    ) -> AppResult<Self> {
        fs::create_dir_all("recordings").map_err(|_| ErrorCode::KConfigFileError)?;
        fs::create_dir_all("snapshots").map_err(|_| ErrorCode::KConfigFileError)?;

        let startup =
            MediaServerStartupConfig::new(config_path, log_output_path, log_level, log_append);
        let factory = MediaServerFactory::new(startup)?;
        let (pipelines, orchestrator) = factory.get_pipeline_orchestration();

        Ok(Self {
            pipelines,
            orchestrator,
        })
    }

    pub fn start(&mut self) -> AppResult<()> {
        self.orchestrator.create_all(&self.pipelines)?;
        self.orchestrator.play_all(&self.pipelines)
    }

    pub fn stop(&mut self) -> AppResult<()> {
        self.orchestrator.stop_all(&self.pipelines)
    }

    pub fn get_pipeline_state(&self, pipeline_name: &str) -> AppResult<String> {
        self.orchestrator.get_state(pipeline_name)
    }
}
