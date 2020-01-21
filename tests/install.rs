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

#[cfg(test)]
#[path = "../tests_utils/mod.rs"]
mod tests_utils;

#[test]
fn test_install_no_init() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!dir.path().join(".protovend.yml").exists());

    let status = command(&dir).arg("install").status().unwrap();

    assert!(!status.success());

    assert!(!dir.path().join(".protovend.yml").exists());
}

#[test]
fn test_install_existing() {
    let dir = tempfile::tempdir().unwrap();

    let status = command(&dir).arg("init").status().unwrap();

    assert!(status.success());
    assert!(dir.path().join(".protovend.yml").exists());

    let status = command(&dir)
        .arg("add")
        .arg("https://github.com/Skyscanner/protovend-test-protos.git")
        .status()
        .unwrap();

    assert!(status.success());

    let status = command(&dir).arg("install").status().unwrap();

    assert!(status.success());

    let mut file = File::open(dir.path().join(".protovend.lock")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    assert!(contents.contains("url: \"https://github.com/Skyscanner/protovend-test-protos.git\""));
    assert!(contents.contains("min_protovend_version"));

    assert!(dir
        .path()
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v1.proto")
        .exists());
    assert!(!dir
        .path()
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v2.proto")
        .exists());
}

// Test ignored as legacy format assumes SSH but the free version of Travis does not support SSH keys
#[ignore]
#[test]
fn test_install_existing_legacy() {
    let legacy_config_contents = "---\
                                  \nmin_protovend_version: 0.1.8\
                                  \nvendor:\
                                  \n  - repo: Skyscanner/protovend-test-protos\
                                  \n    branch: master\
                                  \n    host: github.com";

    let legacy_protovend_config_path =
        tests_utils::fs::write_contents_to_temp_file(legacy_config_contents, ".protovend.yml");

    let dir = legacy_protovend_config_path.parent().unwrap();

    let status = command(&dir).arg("install").status().unwrap();

    assert!(status.success());

    let mut file = File::open(dir.join(".protovend.lock")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    assert!(contents.contains("url: \"https://github.com/Skyscanner/protovend-test-protos.git\""));
    assert!(contents.contains("min_protovend_version"));

    assert!(dir
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v1.proto")
        .exists());
    assert!(!dir
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v2.proto")
        .exists());

    let expected_contents = String::from(
        "---\
         \nmin_protovend_version: 0.1.8\
         \nvendor:\
         \n  - repo: Skyscanner/protovend-test-protos\
         \n    branch: master\
         \n    host: github.com",
    );

    tests_utils::fs::assert_file_contents_eq(
        expected_contents,
        legacy_protovend_config_path.as_path(),
    );
}

#[test]
fn test_install_branch() {
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

#[test]
fn test_install_structured() {
    let dir = tempfile::tempdir().unwrap();

    let status = command(&dir).arg("init").status().unwrap();

    assert!(status.success());
    assert!(dir.path().join(".protovend.yml").exists());

    let status = command(&dir)
        .arg("add")
        .arg("https://github.com/Skyscanner/protovend-test-protos.git")
        .arg("--branch=structured")
        .status()
        .unwrap();

    assert!(status.success());

    let status = command(&dir).arg("install").status().unwrap();

    assert!(status.success());

    let mut file = File::open(dir.path().join(".protovend.lock")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    assert!(contents.contains("url: \"https://github.com/Skyscanner/protovend-test-protos.git\""));
    assert!(contents.contains("min_protovend_version"));

    assert!(dir
        .path()
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v1.proto")
        .exists());
    assert!(dir
        .path()
        .join(
            "./vendor/proto/skyscanner/protovendtestprotos/inner1/inner2/inner-heartbeat-v3.proto"
        )
        .exists());
}

#[test]
fn test_install_not_existing() {
    let dir = tempfile::tempdir().unwrap();

    let status = command(&dir).arg("init").status().unwrap();

    assert!(status.success());
    assert!(dir.path().join(".protovend.yml").exists());

    let status = command(&dir)
        .arg("add")
        .arg("https://github.com/Skyscanner/no-such-repo.git")
        .status()
        .unwrap();

    assert!(status.success());

    let status = command(&dir).arg("install").status().unwrap();

    assert!(!status.success());

    let mut file = File::open(dir.path().join(".protovend.lock")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    assert!(contents.contains("imports: []"));
    assert!(contents.contains("min_protovend_version"));

    assert!(!dir
        .path()
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v1.proto")
        .exists());
    assert!(!dir
        .path()
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v2.proto")
        .exists());
}

#[test]
fn test_install_switch_branch() {
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
    assert!(contents.contains("min_protovend_version"));

    assert!(dir
        .path()
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v1.proto")
        .exists());
    assert!(dir
        .path()
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v2.proto")
        .exists());

    fs::remove_file(dir.path().join(".protovend.lock")).unwrap();

    let mut file = File::create(dir.path().join(".protovend.yml")).unwrap();
    file.write_all(b"min_protovend_version: 0.0.0\nvendor:\n- branch: master\n  url: \"https://github.com/Skyscanner/protovend-test-protos.git\"").unwrap();

    let status = command(&dir).arg("install").status().unwrap();

    assert!(status.success());

    let mut file = File::open(dir.path().join(".protovend.lock")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    assert!(contents.contains("url: \"https://github.com/Skyscanner/protovend-test-protos.git\""));
    assert!(contents.contains("branch: master"));
    assert!(contents.contains("min_protovend_version"));

    assert!(dir
        .path()
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v1.proto")
        .exists());
    assert!(!dir
        .path()
        .join("./vendor/proto/skyscanner/protovendtestprotos/heartbeat-v2.proto")
        .exists());
}
