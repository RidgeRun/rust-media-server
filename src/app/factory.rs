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

use std::path::PathBuf;

use crate::adapters::{
    LoggerFactory, LoggerType, ParserFactory, ParserType, PipelineManagerFactory,
    PipelineManagerType,
};
use crate::app::services::PipelineOrchestrator;
use crate::common::config::{GSTC_ADDRESS, GSTC_PORT};
use crate::common::errors::AppResult;
use crate::common::logging::log_if_error;
use crate::common::pipeline_config::SourcePipelineConfig;
use crate::ports::{IFileParser, LogLevel, SharedLogger};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaServerStartupConfig {
    pub config_path: PathBuf,
    pub log_output_path: PathBuf,
    pub log_level: i32,
    pub log_append: bool,
}

impl MediaServerStartupConfig {
    pub fn new(
        config_path: PathBuf,
        log_output_path: PathBuf,
        log_level: i32,
        log_append: bool,
    ) -> Self {
        Self {
            config_path,
            log_output_path,
            log_level,
            log_append,
        }
    }
}

pub struct MediaServerFactoryContext {
    pipelines: Vec<SourcePipelineConfig>,
    orchestrator: PipelineOrchestrator,
}

pub struct MediaServerFactory {
    context: MediaServerFactoryContext,
}

impl MediaServerFactory {
    pub fn new(startup: MediaServerStartupConfig) -> AppResult<Self> {
        let logger: SharedLogger = LoggerFactory::create(
            LoggerType::File,
            &startup.log_output_path,
            startup.log_append,
        )?;
        logger.set_level(LogLevel::from_i32(startup.log_level))?;
        let parser: Box<dyn IFileParser> = log_if_error(
            &logger,
            ParserFactory::create(ParserType::Yaml, &startup.config_path),
            "failed to create parser",
        )?;
        let pipelines: Vec<SourcePipelineConfig> = log_if_error(
            &logger,
            parser.parse(),
            "failed to parse media server config",
        )?;
        let pipeline_manager = log_if_error(
            &logger,
            PipelineManagerFactory::create(PipelineManagerType::Gstc, GSTC_ADDRESS, &GSTC_PORT),
            "failed to create pipeline manager",
        )?;
        let orchestrator = PipelineOrchestrator::new(pipeline_manager, logger.clone());
        let context = MediaServerFactoryContext {
            pipelines,
            orchestrator,
        };

        Ok(Self { context })
    }

    pub fn get_pipeline_orchestration(self) -> (Vec<SourcePipelineConfig>, PipelineOrchestrator) {
        (self.context.pipelines, self.context.orchestrator)
    }
}
