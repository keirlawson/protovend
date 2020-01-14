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
use tempfile;

mod common;

#[cfg(test)]
#[path = "../tests_utils/mod.rs"]
mod tests_utils;

#[test]
fn test_add_no_init() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!dir.path().join(".protovend.yml").exists());

    let status = command(&dir)
        .arg("add")
        .arg("https://github.com/Skyscanner/protovend-test-protos.git")
        .status()
        .unwrap();

    assert!(!status.success());

    assert!(!dir.path().join(".protovend.yml").exists());
}

#[test]
fn test_add_with_init() {
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

    let expected_contents = String::from(
        "---\
         \nmin_protovend_version: 4.0.0\
         \nvendor:\
         \n  - url: \"https://github.com/Skyscanner/protovend-test-protos.git\"\
         \n    branch: master",
    );

    tests_utils::fs::assert_file_contents_eq(
        expected_contents,
        dir.path().join(".protovend.yml").as_path(),
    );
}

#[test]
fn test_add_to_existing_legacy() {
    let legacy_contents = "---\
                           \nmin_protovend_version: 4.0.0\
                           \nvendor:\
                           \n  - repo: cell-placement/cell-metadata-service\
                           \n    branch: master\
                           \n    host: github.com";

    let legacy_protovend_config_path =
        tests_utils::fs::write_contents_to_temp_file(legacy_contents, ".protovend.yml");

    let dir = legacy_protovend_config_path.parent().unwrap();

    let status = command(&dir)
        .arg("add")
        .arg("https://github.com/Skyscanner/protovend-test-protos.git")
        .status()
        .unwrap();

    assert!(status.success());

    let expected_contents = String::from(
        "---\
         \nmin_protovend_version: 4.0.0\
         \nvendor:\
         \n  - url: \"git@github.com:cell-placement/cell-metadata-service.git\"\
         \n    branch: master\
         \n  - url: \"https://github.com/Skyscanner/protovend-test-protos.git\"\
         \n    branch: master",
    );

    tests_utils::fs::assert_file_contents_eq(
        expected_contents,
        legacy_protovend_config_path.as_path(),
    );
}

#[test]
fn test_add_two() {
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

    let status = command(&dir)
        .arg("add")
        .arg("git@github.com:Skyscanner/protovend-test-protos-fake.git")
        .status()
        .unwrap();

    assert!(status.success());

    let expected_contents = String::from(
        "---\
         \nmin_protovend_version: 4.0.0\
         \nvendor:\
         \n  - url: \"git@github.com:Skyscanner/protovend-test-protos-fake.git\"\
         \n    branch: master\
         \n  - url: \"https://github.com/Skyscanner/protovend-test-protos.git\"\
         \n    branch: master",
    );

    tests_utils::fs::assert_file_contents_eq(
        expected_contents,
        dir.path().join(".protovend.yml").as_path(),
    );
}

#[test]
fn test_add_with_different_hosts() {
    let dir = tempfile::tempdir().unwrap();
    let status = command(&dir).arg("init").status().unwrap();

    assert!(status.success());

    assert!(dir.path().join(".protovend.yml").exists());

    let status = command(&dir)
        .arg("add")
        .arg("git@github.com:Skyscanner/protovend-test-protos2.git")
        .status()
        .unwrap();

    assert!(status.success());

    let expected_contents = String::from(
        "---\
         \nmin_protovend_version: 4.0.0\
         \nvendor:\
         \n  - url: \"git@github.com:Skyscanner/protovend-test-protos2.git\"\
         \n    branch: master",
    );

    tests_utils::fs::assert_file_contents_eq(
        expected_contents,
        dir.path().join(".protovend.yml").as_path(),
    );
}
