mod common;

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use common::{add_test_remote, create_nested_repo, create_test_repo, get_remote_url, write_config};
use predicates::prelude::*;

fn gemote() -> Command {
    cargo_bin_cmd!("gemote")
}

#[test]
fn save_empty_repo() {
    let (dir, _repo) = create_test_repo();

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "save"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Saved"));

    let content = std::fs::read_to_string(dir.path().join(".gemote")).unwrap();
    assert!(content.contains("[settings]"));
}

#[test]
fn save_single_remote() {
    let (dir, repo) = create_test_repo();
    add_test_remote(&repo, "origin", "https://example.com/repo.git", None);

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "save"])
        .assert()
        .success();

    let content = std::fs::read_to_string(dir.path().join(".gemote")).unwrap();
    assert!(content.contains("origin"));
    assert!(content.contains("https://example.com/repo.git"));
}

#[test]
fn save_multiple_remotes() {
    let (dir, repo) = create_test_repo();
    add_test_remote(&repo, "origin", "https://a.com/repo.git", None);
    add_test_remote(&repo, "upstream", "https://b.com/repo.git", None);

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "save"])
        .assert()
        .success();

    let content = std::fs::read_to_string(dir.path().join(".gemote")).unwrap();
    assert!(content.contains("origin"));
    assert!(content.contains("upstream"));
}

#[test]
fn save_with_push_url() {
    let (dir, repo) = create_test_repo();
    add_test_remote(
        &repo,
        "origin",
        "https://example.com/repo.git",
        Some("git@example.com:repo.git"),
    );

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "save"])
        .assert()
        .success();

    let content = std::fs::read_to_string(dir.path().join(".gemote")).unwrap();
    assert!(content.contains("push_url"));
    assert!(content.contains("git@example.com:repo.git"));
}

#[test]
fn save_fails_if_exists() {
    let (dir, _repo) = create_test_repo();
    write_config(dir.path(), "# existing");

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "save"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--force"));
}

#[test]
fn save_force_overwrites() {
    let (dir, repo) = create_test_repo();
    write_config(dir.path(), "# old content");
    add_test_remote(&repo, "origin", "https://example.com/repo.git", None);

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "save", "--force"])
        .assert()
        .success();

    let content = std::fs::read_to_string(dir.path().join(".gemote")).unwrap();
    assert!(!content.contains("# old content"));
    assert!(content.contains("origin"));
}

#[test]
fn save_custom_config_path() {
    let (dir, repo) = create_test_repo();
    add_test_remote(&repo, "origin", "https://example.com/repo.git", None);
    let config_path = dir.path().join("custom.toml");

    gemote()
        .args([
            "--repo",
            dir.path().to_str().unwrap(),
            "--config",
            config_path.to_str().unwrap(),
            "save",
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("origin"));
    // Default .gemote should NOT exist
    assert!(!dir.path().join(".gemote").exists());
}

#[test]
fn save_then_sync_roundtrip() {
    let (dir, repo) = create_test_repo();
    add_test_remote(&repo, "origin", "https://example.com/repo.git", None);
    add_test_remote(
        &repo,
        "upstream",
        "https://upstream.com/repo.git",
        Some("git@upstream.com:repo.git"),
    );

    // Save
    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "save"])
        .assert()
        .success();

    // Delete all remotes
    repo.remote_delete("origin").unwrap();
    repo.remote_delete("upstream").unwrap();
    assert!(repo.remotes().unwrap().is_empty());

    // Sync from saved config
    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "sync"])
        .assert()
        .success();

    // Verify restored
    let (url, _) = get_remote_url(&repo, "origin");
    assert_eq!(url, "https://example.com/repo.git");

    let (url, push_url) = get_remote_url(&repo, "upstream");
    assert_eq!(url, "https://upstream.com/repo.git");
    assert_eq!(push_url.as_deref(), Some("git@upstream.com:repo.git"));
}

#[test]
fn save_recursive_with_nested_repo() {
    let (dir, _repo) = create_test_repo();
    let nested = create_nested_repo(dir.path(), "libs/core");
    nested
        .remote("origin", "https://example.com/core.git")
        .unwrap();

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "save", "-r"])
        .assert()
        .success();

    let content = std::fs::read_to_string(dir.path().join(".gemote")).unwrap();
    assert!(content.contains("[submodules.\"libs/core\".remotes.origin]"));
    assert!(content.contains("https://example.com/core.git"));
}

#[test]
fn save_recursive_no_nested() {
    let (dir, _repo) = create_test_repo();

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "save", "-r"])
        .assert()
        .success();

    let content = std::fs::read_to_string(dir.path().join(".gemote")).unwrap();
    assert!(!content.contains("submodules"));
}

#[test]
fn save_nonrecursive_ignores_nested() {
    let (dir, _repo) = create_test_repo();
    let nested = create_nested_repo(dir.path(), "libs/core");
    nested
        .remote("origin", "https://example.com/core.git")
        .unwrap();

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "save"])
        .assert()
        .success();

    let content = std::fs::read_to_string(dir.path().join(".gemote")).unwrap();
    assert!(!content.contains("submodules"));
}

#[test]
fn save_then_sync_recursive_roundtrip() {
    let (dir, repo) = create_test_repo();
    add_test_remote(&repo, "origin", "https://example.com/repo.git", None);
    let nested = create_nested_repo(dir.path(), "libs/core");
    nested
        .remote("origin", "https://example.com/core.git")
        .unwrap();
    nested
        .remote("upstream", "https://upstream.com/core.git")
        .unwrap();

    // Save recursively
    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "save", "-r"])
        .assert()
        .success();

    // Delete all remotes from nested repo
    nested.remote_delete("origin").unwrap();
    nested.remote_delete("upstream").unwrap();
    assert!(nested.remotes().unwrap().is_empty());

    // Sync recursively to restore
    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "sync", "-r"])
        .assert()
        .success();

    // Verify nested repo's remotes are restored
    let (url, _) = get_remote_url(&nested, "origin");
    assert_eq!(url, "https://example.com/core.git");
    let (url, _) = get_remote_url(&nested, "upstream");
    assert_eq!(url, "https://upstream.com/core.git");
}
