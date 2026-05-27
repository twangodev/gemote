# gemote

CLI for declarative git remote management. Remotes are defined in a `.gemote` TOML file committed to the repo; `gemote sync` makes local remotes match the file, `gemote save` writes current remotes into it.

## Dev

- `cargo test` — unit tests are inline per module; integration tests in `tests/` drive the binary via `assert_cmd`.
- `cargo fmt --check` and `cargo clippy -- -D warnings` must pass (CI gates on both).
- MSRV 1.88.0, edition 2024 — don't reach for newer APIs.

## Where things live

- `cli.rs` — clap arg/subcommand definitions.
- `main.rs` — orchestration + recursive submodule traversal + colored output. No git/diff logic.
- `config.rs` — `GemoteConfig` model and TOML (de)serialization. Recursive: submodules nest as `GemoteConfig`.
- `git.rs` — all `git2` calls: remote ops + repo/submodule discovery.
- `sync.rs` — the diff engine. `compute_diff` is pure; `apply_actions` executes it.
- `error.rs` — `GemoteError`.

The flow is: read config + git state → `compute_diff` → apply or print. Keep `compute_diff` side-effect-free — that's what makes `--dry-run` reliable.
