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

use failure::{format_err, Error};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

lazy_static! {
    static ref REPO: Regex = Regex::new("^/?(.+)\\.git$").unwrap();
    static ref GIT_URL_PATTERN: Regex =
        Regex::new(r"^(?:git|ssh|https?|git)(://|@)(.*)[:/]((.*)/(.*))(\.git)(/?|\#[-\d\w._]+?)$")
            .unwrap();
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct GitUrl(String);

//FIXME move into external lib for reusability
impl GitUrl {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn host(&self) -> String {
        let captures = GIT_URL_PATTERN.captures(self.as_str()).unwrap();

        captures.get(2).unwrap().as_str().to_owned()
    }

    pub fn path(&self) -> String {
        let captures = GIT_URL_PATTERN.captures(self.as_str()).unwrap();

        captures.get(3).unwrap().as_str().to_owned()
    }

    pub fn sanitised_path(&self) -> String {
        self.path()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '/')
            .collect()
    }
}

impl FromStr for GitUrl {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim();
        if GIT_URL_PATTERN.is_match(s) {
            Ok(GitUrl(s.to_owned()))
        } else {
            Err(format_err!("Invalid Git URL"))
        }
    }
}

impl Display for GitUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Host(pub String);

impl Display for Host {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Deserialize, PartialOrd, Eq, Ord, Clone)]
pub struct Repo(pub String);

impl Repo {
    pub fn sanitise(&self) -> Repo {
        Repo(
            self.0
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '/')
                .collect(),
        )
    }
}

impl FromStr for Repo {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Repo(s.to_owned()))
    }
}

impl Display for Repo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_sanitise_strips_and_lowers_characters() {
        let repo = Repo("Test-Python1/a-repo-name".parse().unwrap());
        let expected = Repo("testpython1/areponame".parse().unwrap());
        assert_eq!(expected, repo.sanitise());
    }

    #[test]
    fn test_git_url_from_str() {
        let valid_urls = vec![
            "git@github.skyscannertools.net:keirlawson/protovend.git",
            "git://github.com/ember-cli/ember-cli.git#ff786f9f",
            "git://github.com/ember-cli/ember-cli.git#gh-pages",
            "git://github.com/ember-cli/ember-cli.git#master",
            "git://github.com/ember-cli/ember-cli.git#Quick-Fix",
            "git://github.com/ember-cli/ember-cli.git#quick_fix",
            "git://github.com/ember-cli/ember-cli.git#v0.1.0",
            "git://host.xz/path/to/repo.git/",
            "git://host.xz/~user/path/to/repo.git/",
            "git@192.168.101.127:user/project.git",
            "git@github.com:user/project.git",
            "git@github.com:user/some-project.git",
            "git@github.com:user/some-project.git",
            "git@github.com:user/some_project.git",
            "git@github.com:user/some_project.git",
            "http://192.168.101.127/user/project.git",
            "http://github.com/user/project.git",
            "http://host.xz/path/to/repo.git/",
            "https://192.168.101.127/user/project.git",
            "https://github.com/user/project.git",
            "https://host.xz/path/to/repo.git/",
            "https://username::;*%$:@github.com/username/repository.git",
            "https://username:$fooABC@:@github.com/username/repository.git",
            "https://username:password@github.com/username/repository.git",
            "ssh://host.xz/path/to/repo.git/",
            "ssh://host.xz/path/to/repo.git/",
            "ssh://host.xz/~/path/to/repo.git",
            "ssh://host.xz/~user/path/to/repo.git/",
            "ssh://host.xz:port/path/to/repo.git/",
            "ssh://user@host.xz/path/to/repo.git/",
            "ssh://user@host.xz/path/to/repo.git/",
            "ssh://user@host.xz/~/path/to/repo.git",
            "ssh://user@host.xz/~user/path/to/repo.git/",
            "ssh://user@host.xz:port/path/to/repo.git/",
        ];

        for url in valid_urls.iter() {
            assert!(GitUrl::from_str(url).is_ok())
        }
    }

    #[test]
    fn test_invalid_git_urls() {
        let invalid_urls = vec![
            "/path/to/repo.git/",
            "file:///path/to/repo.git/",
            "file://~/path/to/repo.git/",
            "git@github.com:user/some_project.git/foo",
            "git@github.com:user/some_project.gitfoo",
            "host.xz:/path/to/repo.git/",
            "host.xz:path/to/repo.git",
            "host.xz:~user/path/to/repo.git/",
            "path/to/repo.git/",
            "rsync://host.xz/path/to/repo.git/",
            "user@host.xz:/path/to/repo.git/",
            "user@host.xz:path/to/repo.git",
            "user@host.xz:~user/path/to/repo.git/",
            "~/path/to/repo.git",
        ];

        for url in invalid_urls.iter() {
            assert!(GitUrl::from_str(url).is_err())
        }
    }

    #[test]
    fn test_path_extraction() {
        let url = GitUrl::from_str("https://github.com/user/project.git").unwrap();

        assert_eq!("user/project", url.path());
    }

    #[test]
    fn test_host_extraction() {
        let url = GitUrl::from_str("https://github.com/user/project.git").unwrap();

        assert_eq!("github.com", url.host());
    }
}
