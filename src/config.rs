/*
 * Copyright 2020 Skyscanner Limited.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
*/

use crate::git_url::GitUrl;
use crate::util;
use crate::Result;
use failure::format_err;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

#[cfg(test)]
#[path = "../tests_utils/mod.rs"]
mod tests_utils;

pub const PROTOVEND_YAML: &str = ".protovend.yml";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    pub url: GitUrl,
    pub branch: String,
    pub proto_dir: String,
    pub proto_paths: Vec<String>,
    pub filename_regex: String,
    pub resolve_dependency: bool,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct LegacyDependency {
    pub url: GitUrl,
    pub branch: String,
    pub proto_dir: String,
    pub proto_paths: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProtovendConfig {
    pub min_protovend_version: Version,
    pub vendor: Vec<Dependency>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct LegacyProtovendConfig {
    pub min_protovend_version: Version,
    pub vendor: Vec<LegacyDependency>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct EmptyProtovendConfig {
    pub min_protovend_version: Version,
    pub vendor: (),
}

impl From<EmptyProtovendConfig> for ProtovendConfig {
    fn from(empty: EmptyProtovendConfig) -> Self {
        ProtovendConfig {
            min_protovend_version: empty.min_protovend_version,
            vendor: vec![],
        }
    }
}

impl From<LegacyProtovendConfig> for ProtovendConfig {
    fn from(legacy_config: LegacyProtovendConfig) -> Self {
        ProtovendConfig {
            min_protovend_version: legacy_config.min_protovend_version,
            vendor: legacy_config.vendor.into_iter().map(|d| d.into()).collect(),
        }
    }
}

impl From<LegacyDependency> for Dependency {
    fn from(dep: LegacyDependency) -> Self {
        Dependency {
            url: dep.url.clone(),
            branch: dep.branch,
            proto_dir: dep.proto_dir,
            proto_paths: dep.proto_paths,
            filename_regex: String::from(".*"),
            resolve_dependency: false,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Config {
    Current(ProtovendConfig),
    Legacy(LegacyProtovendConfig),
    Empty(EmptyProtovendConfig),
}

impl ProtovendConfig {
    pub fn write(&mut self) -> Result<()> {
        let f = File::create(Path::new(PROTOVEND_YAML))?;
        self.vendor.sort_by(|a, b| a.url.cmp(&b.url));
        serde_yaml::to_writer(f, &self).map_err(|e| e.into())
    }

    pub fn add_dependency(
        &mut self,
        url: GitUrl,
        branch: String,
        proto_dir: String,
        proto_path: String,
        filename_regex: String,
        resolve_dependency: bool,
    ) -> Result<()> {
        let existing_dep = self.vendor.iter_mut().find(|dep| dep.url == url);

        match existing_dep {
            Some(dep) => {
                if dep.branch != branch {
                    dep.branch = branch.clone();
                    log::info!("Updated {} to use branch {}", url, branch)
                }
                if dep.proto_dir != proto_dir {
                    dep.proto_dir = proto_dir.clone();
                    log::info!("Updated {} to use proto_dir {}", url, proto_dir)
                }
                if dep.filename_regex != filename_regex {
                    dep.filename_regex = filename_regex.clone();
                    log::info!("Updated {} to use filename_regex {}", url, filename_regex)
                }
                if dep.resolve_dependency != resolve_dependency {
                    dep.resolve_dependency = resolve_dependency;
                    log::info!(
                        "Updated {} to use resolve_dependency {}",
                        url,
                        resolve_dependency
                    )
                }
                if dep.proto_paths.contains(&proto_path) {
                    self.write().map(|_| {
                        log::info!(
                            "{}({}) has already added to {}",
                            url,
                            proto_path,
                            PROTOVEND_YAML
                        )
                    })
                } else {
                    dep.proto_paths.push(proto_path.clone());
                    self.write()
                        .map(|_| log::info!("Added proto_path {} to {}", proto_path, url))
                }
            }
            None => {
                let new = Dependency {
                    url: url.clone(),
                    branch,
                    proto_dir,
                    proto_paths: vec![proto_path],
                    filename_regex,
                    resolve_dependency,
                };
                self.vendor.push(new);
                self.write()
                    .map(|_| log::info!("{} added to protovend metadata", url))
            }
        }
    }
}

pub fn init() -> Result<()> {
    let protovend_yaml_path = Path::new(PROTOVEND_YAML);
    if protovend_yaml_path.exists() {
        log::warn!("{} file already exists in project", PROTOVEND_YAML);
        Ok(())
    } else {
        let mut config = ProtovendConfig {
            min_protovend_version: crate::CRATE_VERSION.clone(),
            vendor: Vec::new(),
        };
        config
            .write()
            .map(|_| log::info!("Created {}", PROTOVEND_YAML))
    }
}

pub fn get_config() -> Result<ProtovendConfig> {
    load_config(Path::new(PROTOVEND_YAML))
}

fn load_config(config_file: &Path) -> Result<ProtovendConfig> {
    if config_file.is_file() {
        let f = File::open(config_file)?;
        let config: Config = serde_yaml::from_reader(f)?;

        let config: ProtovendConfig = match config {
            Config::Current(p) => p,
            Config::Legacy(l) => l.into(),
            Config::Empty(e) => e.into(),
        };

        if util::is_valid_version(&config.min_protovend_version) {
            Ok(config)
        } else {
            Err(format_err!("protovend cli version {} is too old for included metadata files. Minimum version must be {}", *crate::CRATE_VERSION, config.min_protovend_version))
        }
    } else {
        Err(format_err!(
            "Project not initialised. Please run 'protovend init'"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correctly_parses_config() {
        let config_contents = "--- \
             \nmin_protovend_version: 0.1.8 \
             \nvendor: \
             \n  - url: git@github.skyscannertools.net:cell-placement/cell-metadata-service.git \
             \n    branch: master \
             \n    proto_dir: proto \
             \n    proto_paths: \
             \n      - path/to \
             \n    filename_regex: ^(a|b) \
             \n    resolve_dependency: true";

        let config_path =
            tests_utils::fs::write_contents_to_temp_file(config_contents, "protovend_config");

        let expected_config = ProtovendConfig {
            min_protovend_version: "0.1.8".parse::<Version>().unwrap(),
            vendor: vec![Dependency {
                url: "git@github.skyscannertools.net:cell-placement/cell-metadata-service.git"
                    .parse::<GitUrl>()
                    .unwrap(),
                branch: String::from("master"),
                proto_dir: String::from("proto"),
                proto_paths: vec![String::from("path/to")],
                filename_regex: String::from("^(a|b)"),
                resolve_dependency: true,
            }],
        };

        let actual_config = load_config(&config_path).unwrap();

        assert_eq!(expected_config, actual_config);
    }

    #[test]
    fn test_correctly_parses_empty_config() {
        let config_contents = "--- \
                               \nmin_protovend_version: 0.1.8 \
                               \nvendor: ";

        let config_path =
            tests_utils::fs::write_contents_to_temp_file(config_contents, "empty_config");

        let expected_config = ProtovendConfig {
            min_protovend_version: "0.1.8".parse::<Version>().unwrap(),
            vendor: vec![],
        };

        let actual_config = load_config(&config_path).unwrap();

        assert_eq!(expected_config, actual_config);
    }

    #[test]
    fn test_config_from_empty_config() {
        let legacy_config = EmptyProtovendConfig {
            min_protovend_version: "0.1.8".parse::<Version>().unwrap(),
            vendor: (),
        };

        let expected_config = ProtovendConfig {
            min_protovend_version: "0.1.8".parse::<Version>().unwrap(),
            vendor: vec![],
        };

        let actual_config = ProtovendConfig::from(legacy_config);

        assert_eq!(expected_config, actual_config);
    }
}
