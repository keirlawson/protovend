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

use crate::git_url::GitUrl;
use crate::{util, Result};
use git2::{build::CheckoutBuilder, Oid, Repository, ResetType};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

mod commands;

pub fn get_latest_commit_sha(url: &GitUrl, branch_name: &str) -> Result<Oid> {
    log::info!(
        "Fetching latest commit hash from {} branch of {}",
        branch_name,
        url
    );
    let repo = get_repo(url, branch_name, "HEAD")?;
    let commit = repo.head()?.peel_to_commit()?;
    Ok(commit.id())
}

pub fn get_repo(url: &GitUrl, branch: &str, revision: &str) -> Result<Repository> {
    let destination_path = get_destination_path(&url);

    if destination_path.exists() {
        log::debug!(
            "Checking out {} under branch {} for revision {}",
            url,
            branch,
            revision
        );
        reset_local_repo_to_commit(&destination_path, branch, revision)
    } else {
        log::debug!("Cloning {} to {}", url, destination_path.display());
        let repo = clone(url.as_str(), &destination_path, branch)?; //FIXME push GitUrl further down
        {
            let commit = repo.revparse_single(revision)?;
            repo.reset(&commit, ResetType::Hard, None)?;
        }
        Ok(repo)
    }
}

fn get_destination_path(url: &GitUrl) -> PathBuf {
    let host = util::to_alpha_num(&url.host());
    let mut destination_path = crate::REPOS_CACHE_DIRECTORY.clone();
    destination_path.push(&host);
    destination_path.push(url.path());
    destination_path
}

fn clone<P: AsRef<Path>>(url: &str, clone_dir: P, branch: &str) -> Result<Repository> {
    fs::create_dir_all(&clone_dir)?;
    commands::clone(&clone_dir, url, branch)?;
    let repo = Repository::open(&clone_dir)?;
    Ok(repo)
}

fn reset_local_repo_to_commit<P: AsRef<Path>>(
    repo_path: P,
    branch: &str,
    revision: &str,
) -> Result<Repository> {
    let repo = Repository::open(&repo_path)?;

    // Ensure all heads and origins are fetched.
    repo.remote_add_fetch("origin", "+refs/heads/*:refs/remotes/origin/*")?;

    // Pull updates for the relevant branch
    commands::fetch(repo_path, branch, "origin")?;

    let branch = &format!("origin/{}", branch);

    // Blast any current changes & checkout actual branch
    {
        let b = repo.resolve_reference_from_short_name(branch)?;
        let obj = repo.revparse_single(branch)?;
        let mut cb = CheckoutBuilder::new();
        cb.remove_untracked(true);
        cb.force();
        repo.checkout_tree(&obj, Some(&mut cb))?;
        repo.set_head(b.name().unwrap())?;
    }

    // Move to latest branch
    {
        let obj = repo.revparse_single(branch)?;
        repo.reset(&obj, ResetType::Hard, None)?;
    }

    // Move to specified revision
    {
        let obj = repo.revparse_single(revision)?;
        repo.reset(&obj, ResetType::Hard, None)?;
    }

    Ok(repo)
}

pub fn get_repo_from_dir(location: &Path) -> Result<GitUrl> {
    let url = commands::get_remote_url(location)?;
    let url = GitUrl::from_str(&url)?;
    Ok(url)
}
