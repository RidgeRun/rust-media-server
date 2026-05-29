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
use std::path::{Path, PathBuf};
use yaml_rust2::{Yaml, YamlLoader};

use crate::common::errors::{AppResult, ErrorCode};
use crate::common::pipeline_config::{FeaturePipelineConfig, SourcePipelineConfig};
use crate::ports::ifileparser::IFileParser;

pub struct YamlParser {
    path: PathBuf,
}

impl YamlParser {
    pub fn new<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
        })
    }

    fn get_str<'a>(node: &'a Yaml, key: &str) -> AppResult<&'a str> {
        node[key].as_str().ok_or(ErrorCode::KConfigFileError)
    }

    fn load_yaml(path: &Path) -> AppResult<Vec<Yaml>> {
        let content = fs::read_to_string(path).map_err(|_| ErrorCode::KConfigFileError)?;
        YamlLoader::load_from_str(&content).map_err(|_| ErrorCode::KConfigFileError)
    }

    fn replace_placeholders(template: &str, pairs: &[(&str, &str)]) -> String {
        let mut resolved = template.to_string();

        for (key, value) in pairs {
            let placeholder = format!("${{{key}}}");
            resolved = resolved.replace(&placeholder, value);
        }

        resolved
    }

    fn has_unresolved_placeholders(text: &str) -> bool {
        let bytes = text.as_bytes();
        let mut i = 0;

        while i + 2 < bytes.len() {
            if bytes[i] == b'$' && bytes[i + 1] == b'{' {
                let mut j = i + 2;

                while j < bytes.len() && bytes[j] != b'}' {
                    j += 1;
                }

                if j < bytes.len() && bytes[j] == b'}' {
                    return true;
                }
            }

            i += 1;
        }

        false
    }
}

impl IFileParser for YamlParser {
    fn parse(&self) -> AppResult<Vec<SourcePipelineConfig>> {
        let docs = Self::load_yaml(&self.path)?;
        let doc = docs.first().ok_or(ErrorCode::KConfigFileError)?;

        let features_node = &doc["features"];
        let sources_node = doc["sources"].as_vec().ok_or(ErrorCode::KConfigFileError)?;

        let base_dir = self.path.parent().ok_or(ErrorCode::KConfigFileError)?;
        let mut configs = Vec::new();

        for source_entry in sources_node {
            let source_rel_path = Self::get_str(source_entry, "file")?;
            let source_path = base_dir.join(source_rel_path);

            let source_docs = Self::load_yaml(&source_path)?;
            let source_doc = source_docs.first().ok_or(ErrorCode::KConfigFileError)?;

            let source = &source_doc["source"];
            let source_enabled = source["enabled"]
                .as_bool()
                .ok_or(ErrorCode::KConfigFileError)?;

            if !source_enabled {
                continue;
            }

            let source_name = Self::get_str(source, "name")?;
            let interpipe_sink = Self::get_str(&source["interpipe"], "interpipe_sink")?;

            let source_pipeline = &source["source_pipeline"];
            let source_pipeline_name = Self::get_str(source_pipeline, "pipeline_id")?;
            let source_pipeline_description = Self::get_str(source_pipeline, "description")?;

            let resolved_source_description = Self::replace_placeholders(
                source_pipeline_description,
                &[("interpipe_sink", interpipe_sink)],
            );

            if Self::has_unresolved_placeholders(&resolved_source_description) {
                return Err(ErrorCode::KConfigFileError);
            }

            let mut feature_pipelines = Vec::new();
            let source_features = source_doc["features"]
                .as_hash()
                .ok_or(ErrorCode::KConfigFileError)?;

            for (feature_key, feature_value) in source_features {
                let feature_name = feature_key.as_str().ok_or(ErrorCode::KConfigFileError)?;
                let enabled = feature_value["enabled"]
                    .as_bool()
                    .ok_or(ErrorCode::KConfigFileError)?;

                if !enabled {
                    continue;
                }

                let feature_file_rel = Self::get_str(features_node, feature_name)?;
                let feature_file_path = base_dir.join(feature_file_rel);
                let feature_docs = Self::load_yaml(&feature_file_path)?;
                let feature_doc = feature_docs.first().ok_or(ErrorCode::KConfigFileError)?;

                let feature_root = &feature_doc[feature_name];
                let template = Self::get_str(feature_root, "pipeline_template")?;
                let pipeline_name = Self::get_str(feature_value, "pipeline_id")?;

                let overrides = feature_value["overrides"]
                    .as_hash()
                    .ok_or(ErrorCode::KConfigFileError)?;

                let mut replacements = vec![("interpipe_sink", interpipe_sink)];
                let mut owned_pairs = Vec::new();

                for (override_key, override_value) in overrides {
                    let key = override_key.as_str().ok_or(ErrorCode::KConfigFileError)?;
                    let value = override_value.as_str().ok_or(ErrorCode::KConfigFileError)?;
                    owned_pairs.push((key.to_string(), value.to_string()));
                }

                for (key, value) in &owned_pairs {
                    replacements.push((key.as_str(), value.as_str()));
                }

                let resolved_feature_description =
                    Self::replace_placeholders(template, &replacements);

                if Self::has_unresolved_placeholders(&resolved_feature_description) {
                    return Err(ErrorCode::KConfigFileError);
                }

                feature_pipelines.push(FeaturePipelineConfig {
                    name: pipeline_name.to_string(),
                    description: resolved_feature_description,
                });
            }

            configs.push(SourcePipelineConfig {
                name: format!("{source_name}:{source_pipeline_name}"),
                description: resolved_source_description,
                features: feature_pipelines,
            });
        }

        Ok(configs)
    }
}
