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

use crate::git_url::{GitUrl, Host, Repo};
use crate::util;
use crate::Result;
use failure::format_err;
use lazy_static::lazy_static;
use log;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

#[cfg(test)]
#[path = "../tests_utils/mod.rs"]
mod tests_utils;

lazy_static! {
    pub static ref PROTOVEND_YAML: PathBuf = PathBuf::from(".protovend.yml");
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    pub url: GitUrl,
    pub branch: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct GithubDependency {
    pub repo: Repo,
    pub branch: String,
    pub host: Host,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProtovendConfig {
    pub min_protovend_version: Version,
    pub vendor: Vec<Dependency>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct LegacyProtovendConfig {
    pub min_protovend_version: Version,
    pub vendor: Vec<GithubDependency>,
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

impl From<GithubDependency> for Dependency {
    fn from(dep: GithubDependency) -> Self {
        let url = format!("git@{}:{}.git", dep.host, dep.repo);
        let url = GitUrl::from_str(url.as_str()).unwrap();
        Dependency {
            url,
            branch: dep.branch,
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
        let f = File::create(PROTOVEND_YAML.as_path())?;
        self.vendor.sort_by(|a, b| a.url.cmp(&b.url));
        serde_yaml::to_writer(f, &self).map_err(|e| e.into())
    }

    pub fn add_dependency(&mut self, url: GitUrl, branch: String) -> Result<()> {
        let existing_dep = self.vendor.iter_mut().find(|dep| dep.url == url);

        match existing_dep {
            Some(dep) => {
                if dep.branch == branch {
                    log::info!(
                        "{} has already added to {}",
                        url,
                        PROTOVEND_YAML.to_string_lossy()
                    );
                    Ok(())
                } else {
                    dep.branch = branch.clone();
                    self.write()
                        .map(|_| log::info!("Updated {} to use branch {}", url, branch))
                }
            }
            None => {
                let new = Dependency {
                    url: url.clone(),
                    branch,
                };
                self.vendor.push(new);
                self.write()
                    .map(|_| log::info!("{} added to protovend metadata", url))
            }
        }
    }
}

pub fn init() -> Result<()> {
    if PROTOVEND_YAML.exists() {
        log::warn!(
            "{} file already exists in project",
            PROTOVEND_YAML.to_string_lossy()
        );
        Ok(())
    } else {
        let mut config = ProtovendConfig {
            min_protovend_version: crate::CRATE_VERSION.clone(),
            vendor: Vec::new(),
        };
        config
            .write()
            .map(|_| log::info!("Created {}", PROTOVEND_YAML.as_path().to_string_lossy()))
    }
}

pub fn get_config() -> Result<ProtovendConfig> {
    load_config(&PROTOVEND_YAML)
}

fn load_config(config_file: &PathBuf) -> Result<ProtovendConfig> {
    if config_file.is_file() {
        let f = File::open(config_file.as_path())?;
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
        let config_contents =
            "--- \
             \nmin_protovend_version: 0.1.8 \
             \nvendor: \
             \n  - url: git@github.skyscannertools.net:cell-placement/cell-metadata-service.git \
             \n    branch: master";

        let config_path =
            tests_utils::fs::write_contents_to_temp_file(config_contents, "protovend_config");

        let expected_config = ProtovendConfig {
            min_protovend_version: Version::from_str("0.1.8").unwrap(),
            vendor: vec![Dependency {
                url: GitUrl::from_str(
                    "git@github.skyscannertools.net:cell-placement/cell-metadata-service.git",
                )
                .unwrap(),
                branch: String::from("master"),
            }],
        };

        let actual_config = load_config(&config_path).unwrap();

        assert_eq!(expected_config, actual_config);
    }

    #[test]
    fn test_correctly_parses_legacy_config() {
        let config_contents = "--- \
                               \nmin_protovend_version: 0.1.8 \
                               \nvendor: \
                               \n  - repo: cell-placement/cell-metadata-service \
                               \n    branch: master \
                               \n    host: github.skyscannertools.net";

        let config_path =
            tests_utils::fs::write_contents_to_temp_file(config_contents, "legacy_config");

        let expected_config = ProtovendConfig {
            min_protovend_version: Version::from_str("0.1.8").unwrap(),
            vendor: vec![Dependency {
                url: GitUrl::from_str(
                    "git@github.skyscannertools.net:cell-placement/cell-metadata-service.git",
                )
                .unwrap(),
                branch: String::from("master"),
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
            min_protovend_version: Version::from_str("0.1.8").unwrap(),
            vendor: vec![],
        };

        let actual_config = load_config(&config_path).unwrap();

        assert_eq!(expected_config, actual_config);
    }

    #[test]
    fn test_config_from_legacy_config() {
        let legacy_config = LegacyProtovendConfig {
            min_protovend_version: Version::from_str("0.1.8").unwrap(),
            vendor: vec![GithubDependency {
                repo: Repo::from_str("cell-placement/cell-metadata-service").unwrap(),
                branch: String::from("master"),
                host: Host {
                    0: String::from("github.skyscannertools.net"),
                },
            }],
        };

        let expected_config = ProtovendConfig {
            min_protovend_version: Version::from_str("0.1.8").unwrap(),
            vendor: vec![Dependency {
                url: GitUrl::from_str(
                    "git@github.skyscannertools.net:cell-placement/cell-metadata-service.git",
                )
                .unwrap(),
                branch: String::from("master"),
            }],
        };

        let actual_config = ProtovendConfig::from(legacy_config);

        assert_eq!(expected_config, actual_config);
    }

    #[test]
    fn test_config_from_empty_config() {
        let legacy_config = EmptyProtovendConfig {
            min_protovend_version: Version::from_str("0.1.8").unwrap(),
            vendor: (),
        };

        let expected_config = ProtovendConfig {
            min_protovend_version: Version::from_str("0.1.8").unwrap(),
            vendor: vec![],
        };

        let actual_config = ProtovendConfig::from(legacy_config);

        assert_eq!(expected_config, actual_config);
    }
}
