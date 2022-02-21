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

use crate::config::Dependency;
use crate::config::ProtovendConfig;
use crate::git;
use crate::git_url::{GitUrl, Host, Repo};
use crate::util;
use crate::{date_compat, Result};
use chrono::{Local, NaiveDateTime};
use failure::format_err;
use lazy_static::lazy_static;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

pub mod vendor;

#[cfg(test)]
#[path = "../tests_utils/mod.rs"]
mod tests_utils;

lazy_static! {
    pub static ref PROTOVEND_LOCK: PathBuf = PathBuf::from(".protovend.lock");
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct Import {
    branch: String,
    commit: String,
    url: GitUrl,
}

#[derive(Deserialize)]
struct GithubImport {
    branch: String,
    commit: String,
    host: Host,
    repo: Repo,
}

impl From<GithubImport> for Import {
    fn from(import: GithubImport) -> Self {
        let url = format!("git@{}:{}.git", import.host, import.repo); //FIXME DRY this up with deps
        let url = GitUrl::from_str(url.as_str()).unwrap();
        Import {
            url,
            branch: import.branch,
            commit: import.commit,
        }
    }
}

impl PartialEq<Dependency> for Import {
    fn eq(&self, other: &Dependency) -> bool {
        self.url == other.url && self.branch == other.branch
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ProtovendLock {
    imports: Vec<Import>,
    min_protovend_version: Version,
    #[serde(with = "date_compat")]
    updated: NaiveDateTime,
}

#[derive(Deserialize)]
pub struct LegacyProtovendLock {
    imports: Vec<GithubImport>,
    min_protovend_version: Version,
    #[serde(with = "date_compat")]
    updated: NaiveDateTime,
}

impl From<LegacyProtovendLock> for ProtovendLock {
    fn from(legacy_config: LegacyProtovendLock) -> Self {
        ProtovendLock {
            min_protovend_version: legacy_config.min_protovend_version,
            imports: legacy_config
                .imports
                .into_iter()
                .map(|d| d.into())
                .collect(),
            updated: legacy_config.updated,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Lock {
    Lock(ProtovendLock),
    Legacy(LegacyProtovendLock),
}

impl ProtovendLock {
    pub fn vendor(&self) -> Result<()> {
        vendor::prepare_output_directory()?;

        for import in self.imports.iter() {
            vendor::vendor_import(import)?;
        }

        Ok(())
    }

    fn write(&mut self) -> Result<()> {
        let f = File::create(PROTOVEND_LOCK.as_path())?;
        self.imports.sort_by(|a, b| a.url.cmp(&b.url));
        self.updated = Local::now().naive_local();
        Ok(serde_yaml::to_writer(f, &self)?)
    }

    fn process_new_imports(&self, deps: Vec<Dependency>) -> Result<Vec<Import>> {
        let (mut entries, added_entries) = diff_lock(deps, self.imports.clone());

        let new_entries: Result<Vec<Import>> = added_entries.into_iter().map(to_import).collect();
        entries.append(&mut new_entries?);

        Ok(entries)
    }

    pub fn update_imports(&mut self, config: ProtovendConfig) -> Result<()> {
        let new_imports = self.process_new_imports(config.vendor)?;
        if new_imports != self.imports {
            self.imports = new_imports;
            self.write()
        } else {
            Ok(())
        }
    }

    pub fn clear_all_imports(&mut self) {
        self.imports.clear()
    }

    pub fn clear_imports(&mut self, repo: GitUrl) {
        self.imports.retain(|import| import.url != repo)
    }
}

pub fn load_lock() -> Result<ProtovendLock> {
    load_lockfile(&PROTOVEND_LOCK)
}

fn load_lockfile(lock_file: &PathBuf) -> Result<ProtovendLock> {
    if lock_file.exists() {
        let f = File::open(lock_file.as_path())?;
        let lock: Lock = serde_yaml::from_reader(f)?;

        let lock: ProtovendLock = match lock {
            Lock::Lock(p) => p,
            Lock::Legacy(l) => l.into(),
        };

        if util::is_valid_version(&lock.min_protovend_version) {
            Ok(lock)
        } else {
            Err(format_err!("protovend cli version {} is too old for included metadata files. Minimum version must be {}", *crate::CRATE_VERSION, lock.min_protovend_version))
        }
    } else {
        Ok(ProtovendLock {
            imports: Vec::new(),
            min_protovend_version: crate::CRATE_VERSION.clone(),
            updated: Local::now().naive_local(),
        })
    }
}

fn to_import(dep: Dependency) -> Result<Import> {
    Ok(Import {
        commit: git::get_latest_commit_sha(&dep.url, &dep.branch)?.to_string(),
        branch: dep.branch,
        url: dep.url,
    })
}

fn diff_lock(
    mut deps: Vec<Dependency>,
    mut imports: Vec<Import>,
) -> (Vec<Import>, Vec<Dependency>) {
    let mut retained_imports = Vec::new();
    deps.retain(|dep| {
        if let Some(position) = imports.iter().position(|import| import == dep) {
            retained_imports.push(imports.remove(position));
            false
        } else {
            true
        }
    });

    (retained_imports, deps)
}

pub fn init() -> Result<()> {
    if PROTOVEND_LOCK.exists() {
        log::warn!(
            "{} file already exists in project",
            PROTOVEND_LOCK.to_string_lossy()
        );
        Ok(())
    } else {
        let mut lock = ProtovendLock {
            imports: Vec::new(),
            min_protovend_version: crate::CRATE_VERSION.clone(),
            updated: Local::now().naive_local(),
        };
        lock.write()
            .map(|_| log::info!("Created {}", PROTOVEND_LOCK.as_path().to_string_lossy()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correctly_parses_legacy_lock() {
        let lock_contents = "--- \
                             \nimports: \
                             \n  - branch: master \
                             \n    commit: a9fef901ae63f689a4180bf8255d16a45baf04a1 \
                             \n    host: github.skyscannertools.net \
                             \n    repo: cell-placement/cell-metadata-service \
                             \nmin_protovend_version: 0.1.8 \
                             \nupdated: \"2019-11-20 15:02:12.330896\"";

        let lock_path = tests_utils::fs::write_contents_to_temp_file(lock_contents, "legacy_lock");

        let expected_lock = ProtovendLock {
            imports: vec![Import {
                branch: String::from("master"),
                commit: String::from("a9fef901ae63f689a4180bf8255d16a45baf04a1"),
                url: GitUrl::from_str(
                    "git@github.skyscannertools.net:cell-placement/cell-metadata-service.git",
                )
                .unwrap(),
            }],
            min_protovend_version: Version::from_str("0.1.8").unwrap(),
            updated: NaiveDateTime::from_str("2019-11-20T15:02:12.330896").unwrap(),
        };

        let actual_lock = load_lockfile(&lock_path).unwrap();

        assert_eq!(expected_lock, actual_lock);
    }

    #[test]
    fn test_correctly_parses_lock() {
        let lock_contents = "--- \
             \nimports: \
             \n  - branch: master \
             \n    commit: a9fef901ae63f689a4180bf8255d16a45baf04a1 \
             \n    url: git@github.skyscannertools.net:cell-placement/cell-metadata-service.git \
             \nmin_protovend_version: 0.1.8 \
             \nupdated: \"2019-11-20 15:02:12.330896\"";

        let lock_path = tests_utils::fs::write_contents_to_temp_file(lock_contents, "lock");

        let expected_lock = ProtovendLock {
            imports: vec![Import {
                branch: String::from("master"),
                commit: String::from("a9fef901ae63f689a4180bf8255d16a45baf04a1"),
                url: GitUrl::from_str(
                    "git@github.skyscannertools.net:cell-placement/cell-metadata-service.git",
                )
                .unwrap(),
            }],
            min_protovend_version: Version::from_str("0.1.8").unwrap(),
            updated: NaiveDateTime::from_str("2019-11-20T15:02:12.330896").unwrap(),
        };

        let actual_lock = load_lockfile(&lock_path).unwrap();

        assert_eq!(expected_lock, actual_lock);
    }
}
