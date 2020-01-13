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

use super::Import;
use crate::Result;
use crate::{check, git, PROTOS_DIRECTORY};
use failure::format_err;
use lazy_static::lazy_static;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

lazy_static! {
    pub static ref PROTOS_OUTPUT_DIRECTORY: PathBuf = PathBuf::from("vendor/proto");
}

pub(super) fn vendor_import(import: &Import) -> Result<()> {
    log::info!(
        "Fetching proto files {} branch from git repo. Current: {}",
        import.branch,
        import.url
    );
    let repo = git::get_repo(&import.url, &import.branch, &import.commit)?;
    let clone_location = repo.workdir().unwrap(); //Can unwrap safely as repository is not bare

    let sanitised_path = import.url.sanitised_path();

    let src_folder = create_src_folder_path(&clone_location, &sanitised_path);
    let dest_folder = create_dest_folder_path(&sanitised_path)?;

    log::info!(
        "calling check with {} and {}",
        clone_location.display(),
        import.url
    );
    check::run_checks(clone_location, &import.url)?;

    find_and_copy_protos(&src_folder, &dest_folder)
}

pub(super) fn prepare_output_directory() -> Result<()> {
    if PROTOS_OUTPUT_DIRECTORY.exists() {
        fs::remove_dir_all(PROTOS_OUTPUT_DIRECTORY.as_path())?;
    }

    fs::create_dir_all(PROTOS_OUTPUT_DIRECTORY.as_path())?;

    Ok(())
}

fn create_dest_folder_path(repo: &str) -> Result<PathBuf> {
    Ok(env::current_dir()?
        .join(PROTOS_OUTPUT_DIRECTORY.as_path())
        .join(repo))
}

fn create_src_folder_path<P: AsRef<Path>>(src_working_dir: P, url_path: &str) -> PathBuf {
    src_working_dir
        .as_ref()
        .join(PROTOS_DIRECTORY.as_path())
        .join(url_path)
}

fn find_and_copy_protos(src_folder: &Path, dest_folder: &Path) -> Result<()> {
    if !src_folder.exists() {
        return Err(format_err!(
            "Cannot find expected directory {}",
            src_folder.display()
        ));
    }

    for entry in WalkDir::new(src_folder) {
        let entry = entry?;
        if entry.metadata()?.is_file() && entry.file_name().to_string_lossy().ends_with(".proto") {
            let src_proto_file = entry.path();
            let relative = src_proto_file.strip_prefix(src_folder)?;
            let dest = dest_folder.join(relative);
            fs::create_dir_all(dest.parent().unwrap())?;

            fs::copy(src_proto_file, &dest)?;

            log::debug!("Copied {} to {}", src_proto_file.display(), dest.display());
        }
    }

    Ok(())
}
