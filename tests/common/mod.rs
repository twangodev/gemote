use std::path::{Path, PathBuf};

use tempfile::TempDir;

pub fn create_test_repo() -> (TempDir, git2::Repository) {
    let dir = TempDir::new().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();
    (dir, repo)
}

pub fn add_test_remote(repo: &git2::Repository, name: &str, url: &str, push_url: Option<&str>) {
    repo.remote(name, url).unwrap();
    if let Some(pu) = push_url {
        repo.remote_set_pushurl(name, Some(pu)).unwrap();
    }
}

pub fn write_config(dir: &Path, content: &str) -> PathBuf {
    let path = dir.join(".gemote");
    std::fs::write(&path, content).unwrap();
    path
}

pub fn get_remote_url(repo: &git2::Repository, name: &str) -> (String, Option<String>) {
    let remote = repo.find_remote(name).unwrap();
    let url = remote.url().unwrap().to_string();
    let push_url = remote.pushurl().map(String::from);
    (url, push_url)
}

pub fn create_nested_repo(parent_dir: &Path, relative_path: &str) -> git2::Repository {
    let nested_path = parent_dir.join(relative_path);
    std::fs::create_dir_all(&nested_path).unwrap();
    git2::Repository::init(&nested_path).unwrap()
}
