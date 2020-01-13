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

use crate::Result;
use crate::{git_url::GitUrl, PROTOS_DIRECTORY};
use failure::format_err;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum ErrorCode {
    P001,
    P002,
}

#[derive(Clone)]
struct CheckResult {
    checked_resource: PathBuf,
    message: String,
    error_code: &'static ErrorCode,
}

impl Display for CheckResult {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{}: {:?} {}",
            self.checked_resource.display(),
            self.error_code,
            self.message
        )
    }
}

pub fn run_checks<P: AsRef<Path>>(project_root: P, url: &GitUrl) -> Result<()> {
    let proto_root_folder = project_root.as_ref().join(PROTOS_DIRECTORY.as_path());
    let project_proto_dir = proto_root_folder.join(url.sanitised_path());
    let relative_proto_dir = project_proto_dir.strip_prefix(&proto_root_folder)?;

    log::info!("Running protovend checks..");

    let results: Vec<Result<Vec<CheckResult>>> = vec![
        check_proto_directory_structure(project_proto_dir.as_path(), proto_root_folder.as_path()),
        check_root_proto_folder_has_no_protos(relative_proto_dir, &proto_root_folder),
    ];
    let results: Result<Vec<Vec<CheckResult>>> = results.into_iter().collect();
    let results = results?.concat();

    report(&results);

    if !results.is_empty() {
        Err(format_err!("Validation errors reported"))
    } else {
        Ok(())
    }
}

fn report(results: &[CheckResult]) {
    for result in results {
        log::error!("{}", result);
    }
}

fn check_root_proto_folder_has_no_protos<P: AsRef<Path>>(
    relative_proto_dir: P,
    proto_root_folder: P,
) -> Result<Vec<CheckResult>> {
    let description = format!(
        ".proto files should not be stored in the root /proto folder; 
                      they should be moved to {}. 
                      If source is from another repo please ask the owners to update",
        relative_proto_dir.as_ref().display()
    );

    let dir = fs::read_dir(&proto_root_folder)?;

    let results: IoResult<Vec<CheckResult>> = dir
        .filter_map(|entry| {
            let res: IoResult<Option<CheckResult>> = entry.and_then(|e| {
                if e.metadata()?.is_file() && e.path().extension() == Some(OsStr::new("proto")) {
                    Ok(Some(CheckResult {
                        checked_resource: proto_root_folder.as_ref().into(),
                        message: description.clone(),
                        error_code: &ErrorCode::P001,
                    }))
                } else {
                    Ok(None)
                }
            });

            res.transpose()
        })
        .collect();

    Ok(results?)
}

fn check_proto_directory_structure<P: AsRef<Path>>(
    project_proto_dir: P,
    proto_root_folder: P,
) -> Result<Vec<CheckResult>> {
    let description = format!(
        "Proto folder structure is not correct; it should contain the directory {}. 
    If source is from another repo please ask the owners to update",
        proto_root_folder.as_ref().display()
    );

    let result = if !project_proto_dir.as_ref().exists() {
        vec![CheckResult {
            checked_resource: proto_root_folder.as_ref().into(),
            message: description,
            error_code: &ErrorCode::P002,
        }]
    } else {
        Vec::new()
    };
    Ok(result)
}
