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

use failure::Error;
use lazy_static::lazy_static;
use semver::Version;
use std::env;
use std::fs;
use std::path::PathBuf;

mod check;
mod config;
mod date_compat;
mod git;
pub mod git_url;
mod lock;
mod util;

lazy_static! {
    static ref CRATE_VERSION: Version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    static ref REPOS_CACHE_DIRECTORY: PathBuf = env::temp_dir().join(".protovend/repos");
    static ref PROTOS_DIRECTORY: PathBuf = PathBuf::from("proto");
}

pub type Result<A> = std::result::Result<A, Error>;

pub fn init() -> Result<()> {
    config::init()?;
    lock::init()
}

pub fn add(url: git_url::GitUrl, branch: String) -> Result<()> {
    let mut config = config::get_config()?;

    config.add_dependency(url, branch)
}

pub fn install() -> Result<()> {
    let config = config::get_config()?;
    let mut lock = lock::load_lock()?;
    lock.update_imports(config)?;
    lock.vendor().map(|_| log_blurb())
}

//FIXME consider doing some sort of matching here?
pub fn update(url: Option<git_url::GitUrl>) -> Result<()> {
    let config = config::get_config()?;
    let mut lock = lock::load_lock()?;

    if let Some(repo) = url {
        lock.clear_imports(repo);
    } else {
        lock.clear_all_imports();
    }

    lock.update_imports(config)?;
    lock.vendor().map(|_| log_blurb())
}

pub fn cleanup() -> Result<()> {
    fs::remove_dir_all(REPOS_CACHE_DIRECTORY.as_path())?;
    Ok(())
}

pub fn lint() -> Result<()> {
    let cwd = env::current_dir()?;
    check::run_checks(&cwd, &git::get_repo_from_dir(cwd.as_path())?)
}

fn log_blurb() {
    log::info!("Next Steps:
Check the following protovend generated files and vendored proto directory (containing .proto files) into source control
  - {}
  - {}
  - {}", config::PROTOVEND_YAML.display(), lock::PROTOVEND_LOCK.display(), lock::vendor::PROTOS_OUTPUT_DIRECTORY.display())
}
