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

use common::command;
use git2::Repository;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile;

mod common;

fn create_with_protos_in_root(path: &Path) {
    fs::create_dir(path.join("proto")).unwrap();
    let mut file = File::create(path.join("proto/test.proto")).unwrap();
    file.write_all(b"test").unwrap();
}

fn create_with_protos_in_correct_location(path: &Path) {
    fs::create_dir_all(path.join("proto/skyscanner/protovend")).unwrap();
    let mut file = File::create(path.join("proto/skyscanner/.proto")).unwrap();
    file.write_all(b"test").unwrap();
}

fn init_git_working_dir(path: &Path) {
    let repo = Repository::init(path).unwrap();
    repo.remote("origin", "git@github.com:Skyscanner/protovend.git")
        .unwrap();
}

#[test]
fn test_cli_exits_with_failure_when_checker_fails() {
    let dir = tempfile::tempdir().unwrap();
    create_with_protos_in_root(dir.path());
    init_git_working_dir(dir.path());

    let status = command(&dir).arg("lint").status().unwrap();

    assert!(!status.success());
}

#[test]
fn test_cli_exits_with_pass_when_checker_passes() {
    let dir = tempfile::tempdir().unwrap();
    create_with_protos_in_correct_location(dir.path());
    init_git_working_dir(dir.path());

    let status = command(&dir).arg("lint").status().unwrap();

    assert!(status.success());
}
