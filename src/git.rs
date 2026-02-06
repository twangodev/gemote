use std::collections::BTreeMap;
use std::path::Path;

use crate::error::GemoteError;

pub struct RemoteInfo {
    pub url: String,
    pub push_url: Option<String>,
}

pub fn open_repo(path: Option<&Path>) -> Result<git2::Repository, GemoteError> {
    match path {
        Some(p) => git2::Repository::open(p).map_err(GemoteError::RepoNotFound),
        None => git2::Repository::discover(".").map_err(GemoteError::RepoNotFound),
    }
}

pub fn list_remotes(repo: &git2::Repository) -> Result<BTreeMap<String, RemoteInfo>, GemoteError> {
    let mut map = BTreeMap::new();
    let remotes = repo.remotes()?;
    for name in remotes.iter().flatten() {
        let remote = repo.find_remote(name)?;
        let url = remote.url().unwrap_or_default().to_string();
        let push_url = remote.pushurl().map(String::from);
        map.insert(
            name.to_string(),
            RemoteInfo {
                url,
                push_url,
            },
        );
    }
    Ok(map)
}

pub fn add_remote(
    repo: &git2::Repository,
    name: &str,
    url: &str,
    push_url: Option<&str>,
) -> Result<(), GemoteError> {
    repo.remote(name, url)?;
    if let Some(push) = push_url {
        repo.remote_set_pushurl(name, Some(push))?;
    }
    Ok(())
}

pub fn update_remote_url(
    repo: &git2::Repository,
    name: &str,
    url: &str,
) -> Result<(), GemoteError> {
    repo.remote_set_url(name, url)?;
    Ok(())
}

pub fn update_remote_push_url(
    repo: &git2::Repository,
    name: &str,
    push_url: Option<&str>,
) -> Result<(), GemoteError> {
    repo.remote_set_pushurl(name, push_url)?;
    Ok(())
}

pub fn remove_remote(repo: &git2::Repository, name: &str) -> Result<(), GemoteError> {
    repo.remote_delete(name)?;
    Ok(())
}
