use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use path_slash::PathExt as _;

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

pub struct SubRepoInfo {
    pub path: String,
    pub repo: git2::Repository,
}

pub fn list_submodules(repo: &git2::Repository) -> Result<Vec<SubRepoInfo>, GemoteError> {
    let mut result = Vec::new();
    let submodules = repo.submodules()?;
    for sub in submodules {
        let name = sub.name().unwrap_or_default().to_string();
        match sub.open() {
            Ok(sub_repo) => {
                result.push(SubRepoInfo {
                    path: name,
                    repo: sub_repo,
                });
            }
            Err(e) => {
                eprintln!(
                    "warning: skipping uninitialized submodule '{}': {}",
                    name, e
                );
            }
        }
    }
    Ok(result)
}

pub fn discover_nested_repos(
    repo_root: &Path,
    known_paths: &BTreeSet<String>,
) -> Result<Vec<SubRepoInfo>, GemoteError> {
    let mut result = Vec::new();
    discover_nested_repos_recursive(repo_root, repo_root, known_paths, &mut result)?;
    Ok(result)
}

fn discover_nested_repos_recursive(
    base: &Path,
    dir: &Path,
    known_paths: &BTreeSet<String>,
    result: &mut Vec<SubRepoInfo>,
) -> Result<(), GemoteError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };
    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        // Skip hidden directories (including .git)
        if name_str.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_slash_lossy()
            .into_owned();
        // Skip known submodule paths
        if known_paths.contains(&rel) {
            continue;
        }
        // Check if this directory is a git repo
        if path.join(".git").exists() {
            match git2::Repository::open(&path) {
                Ok(repo) => {
                    result.push(SubRepoInfo { path: rel, repo });
                }
                Err(e) => {
                    eprintln!(
                        "warning: could not open nested repo '{}': {}",
                        path.display(),
                        e
                    );
                }
            }
            // Don't recurse into nested repos — they are their own boundary
            continue;
        }
        // Recurse into subdirectory
        discover_nested_repos_recursive(base, &path, known_paths, result)?;
    }
    Ok(())
}

pub fn collect_all_repos(
    repo: &git2::Repository,
    repo_root: &Path,
) -> Result<Vec<SubRepoInfo>, GemoteError> {
    let submodules = list_submodules(repo)?;
    let known: BTreeSet<String> = submodules.iter().map(|s| s.path.clone()).collect();
    let nested = discover_nested_repos(repo_root, &known)?;

    let mut all = submodules;
    all.extend(nested);
    // Deduplicate by path and sort
    let mut seen = BTreeSet::new();
    all.retain(|info| seen.insert(info.path.clone()));
    all.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(all)
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

    #[test]
    fn list_submodules_empty() {
        let (_dir, repo) = test_repo();
        let subs = list_submodules(&repo).unwrap();
        assert!(subs.is_empty());
    }

    #[test]
    fn discover_nested_repos_empty() {
        let dir = TempDir::new().unwrap();
        git2::Repository::init(dir.path()).unwrap();
        let known = BTreeSet::new();
        let nested = discover_nested_repos(dir.path(), &known).unwrap();
        assert!(nested.is_empty());
    }

    #[test]
    fn discover_nested_repos_finds_repo() {
        let dir = TempDir::new().unwrap();
        git2::Repository::init(dir.path()).unwrap();
        // Create a nested repo
        let nested_path = dir.path().join("libs").join("core");
        std::fs::create_dir_all(&nested_path).unwrap();
        git2::Repository::init(&nested_path).unwrap();

        let known = BTreeSet::new();
        let nested = discover_nested_repos(dir.path(), &known).unwrap();
        assert_eq!(nested.len(), 1);
        assert_eq!(nested[0].path, "libs/core");
    }

    #[test]
    fn discover_nested_repos_skips_known() {
        let dir = TempDir::new().unwrap();
        git2::Repository::init(dir.path()).unwrap();
        let nested_path = dir.path().join("libs").join("core");
        std::fs::create_dir_all(&nested_path).unwrap();
        git2::Repository::init(&nested_path).unwrap();

        let mut known = BTreeSet::new();
        known.insert("libs/core".to_string());
        let nested = discover_nested_repos(dir.path(), &known).unwrap();
        assert!(nested.is_empty());
    }

    #[test]
    fn discover_nested_repos_skips_hidden() {
        let dir = TempDir::new().unwrap();
        git2::Repository::init(dir.path()).unwrap();
        // Create a hidden dir with a git repo inside — should be skipped
        let hidden_path = dir.path().join(".hidden").join("repo");
        std::fs::create_dir_all(&hidden_path).unwrap();
        git2::Repository::init(&hidden_path).unwrap();

        let known = BTreeSet::new();
        let nested = discover_nested_repos(dir.path(), &known).unwrap();
        assert!(nested.is_empty());
    }

    #[test]
    fn collect_all_repos_empty() {
        let (dir, repo) = test_repo();
        let all = collect_all_repos(&repo, dir.path()).unwrap();
        assert!(all.is_empty());
    }

    #[test]
    fn collect_all_repos_discovers_nested() {
        let (dir, repo) = test_repo();
        let nested_path = dir.path().join("vendor").join("lib");
        std::fs::create_dir_all(&nested_path).unwrap();
        git2::Repository::init(&nested_path).unwrap();

        let all = collect_all_repos(&repo, dir.path()).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].path, "vendor/lib");
    }

    #[test]
    fn discover_nested_repos_multiple() {
        let dir = TempDir::new().unwrap();
        git2::Repository::init(dir.path()).unwrap();

        // Create two nested repos at different depths
        let shallow = dir.path().join("libs").join("core");
        std::fs::create_dir_all(&shallow).unwrap();
        git2::Repository::init(&shallow).unwrap();

        let deep = dir.path().join("vendor").join("deps").join("util");
        std::fs::create_dir_all(&deep).unwrap();
        git2::Repository::init(&deep).unwrap();

        let known = BTreeSet::new();
        let nested = discover_nested_repos(dir.path(), &known).unwrap();
        assert_eq!(nested.len(), 2);

        let paths: Vec<&str> = nested.iter().map(|s| s.path.as_str()).collect();
        assert!(paths.contains(&"libs/core"));
        assert!(paths.contains(&"vendor/deps/util"));
    }

    #[test]
    fn discover_nested_repos_stops_at_git_boundary() {
        let dir = TempDir::new().unwrap();
        git2::Repository::init(dir.path()).unwrap();

        // Create an outer nested repo
        let outer = dir.path().join("libs").join("outer");
        std::fs::create_dir_all(&outer).unwrap();
        git2::Repository::init(&outer).unwrap();

        // Create an inner repo inside the outer one — should NOT be found
        let inner = outer.join("sub").join("inner");
        std::fs::create_dir_all(&inner).unwrap();
        git2::Repository::init(&inner).unwrap();

        let known = BTreeSet::new();
        let nested = discover_nested_repos(dir.path(), &known).unwrap();
        assert_eq!(nested.len(), 1);
        assert_eq!(nested[0].path, "libs/outer");
    }

    #[test]
    fn collect_all_repos_deduplicates() {
        let (dir, repo) = test_repo();

        // Create a nested repo that could be found by discovery
        let nested_path = dir.path().join("libs").join("core");
        std::fs::create_dir_all(&nested_path).unwrap();
        git2::Repository::init(&nested_path).unwrap();

        // collect_all_repos merges submodules (empty here) + discovered,
        // then deduplicates — verify no duplicates in output
        let all = collect_all_repos(&repo, dir.path()).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].path, "libs/core");

        // Verify paths are unique
        let paths: BTreeSet<String> = all.iter().map(|s| s.path.clone()).collect();
        assert_eq!(paths.len(), all.len());
    }
}
