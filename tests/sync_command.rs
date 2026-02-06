mod common;

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use common::{add_test_remote, create_test_repo, get_remote_url, write_config};
use predicates::prelude::*;

fn gemote() -> Command {
    cargo_bin_cmd!("gemote")
}

#[test]
fn sync_no_config() {
    let (dir, _repo) = create_test_repo();

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "sync"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("config"));
}

#[test]
fn sync_adds_missing_remote() {
    let (dir, repo) = create_test_repo();
    write_config(
        dir.path(),
        r#"
[remotes.origin]
url = "https://example.com/repo.git"
"#,
    );

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "sync"])
        .assert()
        .success()
        .stdout(predicate::str::contains("add"));

    let (url, _) = get_remote_url(&repo, "origin");
    assert_eq!(url, "https://example.com/repo.git");
}

#[test]
fn sync_adds_with_push_url() {
    let (dir, repo) = create_test_repo();
    write_config(
        dir.path(),
        r#"
[remotes.origin]
url = "https://example.com/repo.git"
push_url = "git@example.com:repo.git"
"#,
    );

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "sync"])
        .assert()
        .success();

    let (url, push_url) = get_remote_url(&repo, "origin");
    assert_eq!(url, "https://example.com/repo.git");
    assert_eq!(push_url.as_deref(), Some("git@example.com:repo.git"));
}

#[test]
fn sync_updates_url() {
    let (dir, repo) = create_test_repo();
    add_test_remote(&repo, "origin", "https://old.com/repo.git", None);
    write_config(
        dir.path(),
        r#"
[remotes.origin]
url = "https://new.com/repo.git"
"#,
    );

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "sync"])
        .assert()
        .success()
        .stdout(predicate::str::contains("update"));

    let (url, _) = get_remote_url(&repo, "origin");
    assert_eq!(url, "https://new.com/repo.git");
}

#[test]
fn sync_updates_push_url() {
    let (dir, repo) = create_test_repo();
    add_test_remote(&repo, "origin", "https://example.com/repo.git", None);
    write_config(
        dir.path(),
        r#"
[remotes.origin]
url = "https://example.com/repo.git"
push_url = "git@example.com:repo.git"
"#,
    );

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "sync"])
        .assert()
        .success();

    let (_, push_url) = get_remote_url(&repo, "origin");
    assert_eq!(push_url.as_deref(), Some("git@example.com:repo.git"));
}

#[test]
fn sync_already_in_sync() {
    let (dir, repo) = create_test_repo();
    add_test_remote(&repo, "origin", "https://example.com/repo.git", None);
    write_config(
        dir.path(),
        r#"
[remotes.origin]
url = "https://example.com/repo.git"
"#,
    );

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "sync"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Already in sync"));
}

#[test]
fn sync_dry_run_no_apply() {
    let (dir, repo) = create_test_repo();
    write_config(
        dir.path(),
        r#"
[remotes.origin]
url = "https://example.com/repo.git"
"#,
    );

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "sync", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dry run"));

    // Remote should NOT have been added
    assert!(repo.find_remote("origin").is_err());
}

#[test]
fn sync_extra_ignore() {
    let (dir, repo) = create_test_repo();
    add_test_remote(&repo, "extra", "https://extra.com/repo.git", None);
    write_config(
        dir.path(),
        r#"
[settings]
extra_remotes = "ignore"
"#,
    );

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "sync"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Already in sync"));

    // extra remote should still exist
    assert!(repo.find_remote("extra").is_ok());
}

#[test]
fn sync_extra_warn() {
    let (dir, repo) = create_test_repo();
    add_test_remote(&repo, "extra", "https://extra.com/repo.git", None);
    write_config(
        dir.path(),
        r#"
[settings]
extra_remotes = "warn"
"#,
    );

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "sync"])
        .assert()
        .success()
        .stderr(predicate::str::contains("warning"));

    // extra remote should still exist
    assert!(repo.find_remote("extra").is_ok());
}

#[test]
fn sync_extra_remove() {
    let (dir, repo) = create_test_repo();
    add_test_remote(&repo, "extra", "https://extra.com/repo.git", None);
    write_config(
        dir.path(),
        r#"
[settings]
extra_remotes = "remove"
"#,
    );

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "sync"])
        .assert()
        .success()
        .stdout(predicate::str::contains("remove"));

    // extra remote should be gone
    assert!(repo.find_remote("extra").is_err());
}

#[test]
fn sync_custom_config_path() {
    let (dir, repo) = create_test_repo();
    let config_path = dir.path().join("custom-config.toml");
    std::fs::write(
        &config_path,
        r#"
[remotes.origin]
url = "https://example.com/repo.git"
"#,
    )
    .unwrap();

    gemote()
        .args([
            "--repo",
            dir.path().to_str().unwrap(),
            "--config",
            config_path.to_str().unwrap(),
            "sync",
        ])
        .assert()
        .success();

    let (url, _) = get_remote_url(&repo, "origin");
    assert_eq!(url, "https://example.com/repo.git");
}

#[test]
fn sync_not_a_repo() {
    let dir = tempfile::TempDir::new().unwrap();

    gemote()
        .args(["--repo", dir.path().to_str().unwrap(), "sync"])
        .assert()
        .failure();
}
