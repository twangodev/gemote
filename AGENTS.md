# AGENTS.md

## Project

`gemote` is a Rust CLI for declarative git remote management. Users commit a `.gemote` TOML file describing the desired remotes, and the tool syncs local git remotes to match (or saves current state into the config). The repo dogfoods itself: its own `.gemote` defines its remotes.

## Common commands

```sh
cargo build                      # debug build
cargo test                       # unit + integration tests
cargo test <name>                # filter by substring
cargo test --test sync_command   # run a single integration test binary
cargo fmt --check                # formatting
cargo clippy -- -D warnings      # lints
cargo run -- sync --dry-run      # run the CLI locally
```

For MSRV checks, coverage, release, Docker, and the full CI matrix, see `.github/workflows/rust.yml` and `.github/targets.json`.

## Architecture

Thin CLI shell over a pure diff/apply core:

1. **`cli.rs`** — clap-derived `Cli` + `Commands` enum (`Sync`, `Save`, `Completions`). Global flags `--config` and `--repo` apply to all subcommands.
2. **`git.rs`** — `git2` wrappers plus submodule / nested-repo discovery (`list_submodules`, `discover_nested_repos`, `collect_all_repos`). Uninitialized submodules and unreadable directories warn rather than error. Paths use forward slashes via `path-slash` for cross-platform TOML stability.
3. **`config.rs`** — serde/TOML schema. `GemoteConfig` is recursive: `submodules: BTreeMap<String, GemoteConfig>` lets the same structure describe nested repos.
4. **`sync.rs`** — the core. `compute_diff(config, local) -> Vec<SyncAction>` is a **pure function** producing `Add`/`UpdateUrl`/`UpdatePushUrl`/`Remove`. `apply_actions` executes them. This separation is why `--dry-run` just skips `apply_actions`; don't collapse the two.
5. **`main.rs`** — orchestration. Recursive mode walks submodules/nested repos, keying into `cfg.submodules` by path.

### Invariants

- `extra_remotes` (`ignore`/`warn`/`remove`) only affects local remotes **not** in config; remotes in config are always added/updated.
- Nested repo discovery stops at git boundaries — a nested repo's subdirectories are its own concern.
- `list_submodules` results take precedence over discovered nested repos; `collect_all_repos` deduplicates by path.

## Testing

- **Unit tests** inline via `#[cfg(test)] mod tests`, using `tempfile::TempDir` + `git2::Repository::init`.
- **Integration tests** in `tests/` drive the binary via `assert_cmd`; prefer the helpers in `tests/common/mod.rs` over open-coded setup.
- On macOS, `git2::Repository::workdir().canonicalize()` is needed to resolve the `/var` → `/private/var` symlink when comparing paths.