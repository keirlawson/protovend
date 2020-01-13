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

#[cfg(test)]
#[allow(dead_code)]
pub mod fs {
    use std::path::Path;
    use std::str::FromStr;
    use std::{env, fs, path::PathBuf};

    pub fn write_contents_to_temp_file(contents: &str, filename: &str) -> PathBuf {
        let temp_file_path = format!("{}/{}", env::temp_dir().to_str().unwrap(), filename);
        fs::remove_file(temp_file_path.as_str()).unwrap_or_default();
        fs::write(temp_file_path.as_str(), contents).unwrap();
        PathBuf::from_str(temp_file_path.as_str()).unwrap()
    }

    pub fn assert_file_contents_eq(expected: String, filepath: &Path) {
        assert_eq!(expected, load_file_contents(filepath))
    }

    fn load_file_contents(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }
}
