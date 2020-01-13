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
fn test_init_fresh() {
    let dir = tempfile::tempdir().unwrap();

    assert!(!dir.path().join(".protovend.yml").exists());

    let status = command(&dir).arg("init").status().unwrap();

    assert!(status.success());
    assert!(dir.path().join(".protovend.yml").exists());

    tests_utils::fs::assert_file_contents_eq(
        String::from("---\nmin_protovend_version: 4.0.0\nvendor: []"),
        dir.path().join(".protovend.yml").as_path(),
    );
}

//FIXME consider assert_fs to make this simpler
#[test]
fn test_init_does_not_overwrite() {
    let foobar_protovend_path =
        tests_utils::fs::write_contents_to_temp_file("foobar", ".protovend.yml");

    let dir = foobar_protovend_path.parent().unwrap();

    assert!(dir.join(".protovend.yml").exists());

    let status = command(&dir).arg("init").status().unwrap();
    assert!(status.success());

    tests_utils::fs::assert_file_contents_eq(
        String::from("foobar"),
        foobar_protovend_path.as_path(),
    )
}
