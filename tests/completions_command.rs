use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

fn gemote() -> Command {
    cargo_bin_cmd!("gemote")
}

#[test]
fn completions_bash_produces_output() {
    gemote()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("gemote"));
}

#[test]
fn completions_zsh_produces_output() {
    gemote()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("gemote"));
}

#[test]
fn completions_fish_produces_output() {
    gemote()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("gemote"));
}

#[test]
fn completions_invalid_shell() {
    gemote()
        .args(["completions", "nushell"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}
