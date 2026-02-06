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
        map.insert(name.to_string(), RemoteInfo { url, push_url });
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_repo() -> (TempDir, git2::Repository) {
        let dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        (dir, repo)
    }

    #[test]
    fn open_repo_with_path() {
        let (dir, _) = test_repo();
        let repo = open_repo(Some(dir.path())).unwrap();
        // Canonicalize to handle macOS /var -> /private/var symlink
        let expected = dir.path().canonicalize().unwrap();
        let actual = repo.workdir().unwrap().canonicalize().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn open_repo_not_found() {
        let result = open_repo(Some(Path::new("/nonexistent/repo")));
        assert!(matches!(result, Err(GemoteError::RepoNotFound(_))));
    }

    #[test]
    fn list_remotes_empty() {
        let (_dir, repo) = test_repo();
        let remotes = list_remotes(&repo).unwrap();
        assert!(remotes.is_empty());
    }

    #[test]
    fn list_remotes_single() {
        let (_dir, repo) = test_repo();
        repo.remote("origin", "https://example.com/repo.git")
            .unwrap();

        let remotes = list_remotes(&repo).unwrap();
        assert_eq!(remotes.len(), 1);
        assert_eq!(remotes["origin"].url, "https://example.com/repo.git");
        assert!(remotes["origin"].push_url.is_none());
    }

    #[test]
    fn list_remotes_multiple() {
        let (_dir, repo) = test_repo();
        repo.remote("origin", "https://a.com/repo.git").unwrap();
        repo.remote("upstream", "https://b.com/repo.git").unwrap();

        let remotes = list_remotes(&repo).unwrap();
        assert_eq!(remotes.len(), 2);
        let keys: Vec<_> = remotes.keys().collect();
        assert_eq!(keys, vec!["origin", "upstream"]);
    }

    #[test]
    fn list_remotes_with_push_url() {
        let (_dir, repo) = test_repo();
        repo.remote("origin", "https://example.com/repo.git")
            .unwrap();
        repo.remote_set_pushurl("origin", Some("git@example.com:repo.git"))
            .unwrap();

        let remotes = list_remotes(&repo).unwrap();
        assert_eq!(
            remotes["origin"].push_url.as_deref(),
            Some("git@example.com:repo.git")
        );
    }

    #[test]
    fn add_remote_basic() {
        let (_dir, repo) = test_repo();
        add_remote(&repo, "origin", "https://example.com/repo.git", None).unwrap();

        let remote = repo.find_remote("origin").unwrap();
        assert_eq!(remote.url().unwrap(), "https://example.com/repo.git");
        assert!(remote.pushurl().is_none());
    }

    #[test]
    fn add_remote_with_push_url() {
        let (_dir, repo) = test_repo();
        add_remote(
            &repo,
            "origin",
            "https://example.com/repo.git",
            Some("git@example.com:repo.git"),
        )
        .unwrap();

        let remote = repo.find_remote("origin").unwrap();
        assert_eq!(remote.url().unwrap(), "https://example.com/repo.git");
        assert_eq!(remote.pushurl().unwrap(), "git@example.com:repo.git");
    }

    #[test]
    fn add_remote_duplicate() {
        let (_dir, repo) = test_repo();
        add_remote(&repo, "origin", "https://example.com/repo.git", None).unwrap();
        let result = add_remote(&repo, "origin", "https://other.com/repo.git", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_remote_url() {
        let (_dir, repo) = test_repo();
        repo.remote("origin", "https://old.com/repo.git").unwrap();

        update_remote_url(&repo, "origin", "https://new.com/repo.git").unwrap();

        let remote = repo.find_remote("origin").unwrap();
        assert_eq!(remote.url().unwrap(), "https://new.com/repo.git");
    }

    #[test]
    fn update_push_url_set() {
        let (_dir, repo) = test_repo();
        repo.remote("origin", "https://example.com/repo.git")
            .unwrap();

        update_remote_push_url(&repo, "origin", Some("git@example.com:repo.git")).unwrap();

        let remote = repo.find_remote("origin").unwrap();
        assert_eq!(remote.pushurl().unwrap(), "git@example.com:repo.git");
    }

    #[test]
    fn update_push_url_clear() {
        let (_dir, repo) = test_repo();
        repo.remote("origin", "https://example.com/repo.git")
            .unwrap();
        repo.remote_set_pushurl("origin", Some("git@example.com:repo.git"))
            .unwrap();

        update_remote_push_url(&repo, "origin", None).unwrap();

        let remote = repo.find_remote("origin").unwrap();
        assert!(remote.pushurl().is_none());
    }

    #[test]
    fn test_remove_remote() {
        let (_dir, repo) = test_repo();
        repo.remote("origin", "https://example.com/repo.git")
            .unwrap();

        remove_remote(&repo, "origin").unwrap();

        assert!(repo.find_remote("origin").is_err());
    }

    #[test]
    fn remove_remote_nonexistent() {
        let (_dir, repo) = test_repo();
        let result = remove_remote(&repo, "nonexistent");
        assert!(result.is_err());
    }
}
