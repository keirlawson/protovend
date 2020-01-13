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
use failure::format_err;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn fetch<P: AsRef<Path>>(cwd: P, branch_name: &str, remote_name: &str) -> Result<()> {
    let status = Command::new("git")
        .current_dir(cwd)
        .arg("fetch")
        .arg(remote_name)
        .arg(branch_name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(format_err!(
            "Git fetch failed with code {:?}",
            status.code()
        ))
    }
}

pub fn clone<P: AsRef<Path>>(cwd: P, url: &str, branch: &str) -> Result<()> {
    let output = Command::new("git")
        .current_dir(cwd)
        .arg("clone")
        .arg(url)
        .arg("--branch")
        .arg(branch)
        .arg(".")
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        dbg!(&output);
        Err(format_err!(
            "Git clone failed with code {:?}",
            output.status.code()
        ))
    }
}

pub fn get_remote_url<P: AsRef<Path>>(cwd: P) -> Result<String> {
    let output = Command::new("git")
        .current_dir(cwd)
        .arg("ls-remote")
        .arg("--get-url")
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(format_err!(
            "Git ls-remote failed with code {:?}",
            output.status.code()
        ))
    }
}
