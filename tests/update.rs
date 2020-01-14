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
use std::fs::{self, File};
use std::io::{Read, Write};
use tempfile;

mod common;

#[test]
fn test_update_no_init() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!dir.path().join(".protovend.yml").exists());

    let status = command(&dir).arg("update").status().unwrap();

    assert!(!status.success());

    assert!(!dir.path().join(".protovend.yml").exists());
}

#[test]
fn test_install_override_commit_hash() {
    let dir = tempfile::tempdir().unwrap();

    let status = command(&dir).arg("init").status().unwrap();

    assert!(status.success());
    assert!(dir.path().join(".protovend.yml").exists());

    let status = command(&dir)
        .arg("add")
        .arg("https://github.com/Skyscanner/protovend-test-protos.git")
        .arg("--branch=branch-2")
        .status()
        .unwrap();

    assert!(status.success());

    let status = command(&dir).arg("install").status().unwrap();

    assert!(status.success());

    let mut file = File::open(dir.path().join(".protovend.lock")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    assert!(contents.contains("url: \"https://github.com/Skyscanner/protovend-test-protos.git\""));
    assert!(contents.contains("branch: branch-2"));

    assert!(dir
        .path()
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v1.proto")
        .exists());
    assert!(dir
        .path()
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v2.proto")
        .exists());

    fs::remove_file(
        dir.path()
            .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v1.proto"),
    )
    .unwrap();
    fs::remove_file(
        dir.path()
            .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v2.proto"),
    )
    .unwrap();

    assert!(dir.path().join(".protovend.yml").exists());

    // Now change lock file to simulate older vendored version

    let mut file = File::create(dir.path().join(".protovend.lock")).unwrap();
    file.write_all(
        b"imports:
- branch: branch-2
  commit: thisIsAnOldHash
  repo: skyscanner/protovend-test-protos
  host: github.skyscannertools.net
min_protovend_version: 0.0.0
updated: 2017-08-14 17:15:13.549503",
    )
    .unwrap();

    let mut file = File::open(dir.path().join(".protovend.lock")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let status = command(&dir).arg("update").status().unwrap();

    assert!(status.success());

    let mut file = File::open(dir.path().join(".protovend.lock")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    assert!(contents.contains("url: \"https://github.com/Skyscanner/protovend-test-protos.git\""));
    assert!(contents.contains("branch: branch-2"));
    assert!(!contents.contains("thisIsAnOldHash"));
    assert!(contents.contains("min_protovend_version"));

    assert!(dir
        .path()
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v1.proto")
        .exists());
    assert!(dir
        .path()
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v2.proto")
        .exists());
}
