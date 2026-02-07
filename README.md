# gemote

[![Crates.io](https://img.shields.io/crates/v/gemote)](https://crates.io/crates/gemote)
[![CI](https://img.shields.io/github/actions/workflow/status/twangodev/gemote/rust.yml)](https://github.com/twangodev/gemote/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/twangodev/gemote/blob/main/LICENSE)

Declarative git remote management. Define your remotes in a `.gemote` file, commit it, and keep the whole team in sync.

## Install

### From crates.io

```sh
cargo install gemote
```

### From source

```sh
cargo install --path .
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/twangodev/gemote/releases). Binaries are available for:

### Docker

```sh
docker run --rm -v "$(pwd):/repo" -w /repo ghcr.io/twangodev/gemote sync
```

## Usage

### `gemote save`

Write your current local remotes into a `.gemote` file:

```sh
gemote save
gemote save -f            # replace existing .gemote (--force)
gemote save -r            # recursive mode (--recursive)
```

### `gemote sync`

Set your local remotes to match the `.gemote` config:

```sh
gemote sync
gemote sync --dry-run     # preview changes without applying
gemote sync -r            # recursive mode (--recursive)
```

### Global flags

```
--config <path>   Path to config file (default: .gemote at repo root)
--repo <path>     Path to git repository (default: discovered from cwd)
```

## Config format

`.gemote` uses TOML:

```toml
[settings]
# What to do with local remotes not in this file: "ignore" (default), "warn", "remove"
extra_remotes = "ignore"

[remotes.origin]
url = "git@github.com:org/repo.git"

[remotes.upstream]
url = "git@github.com:upstream/repo.git"
push_url = "git@github.com:you/repo.git"  # optional, only if push URL differs
```

### Recursive / submodule config

When using `-r`/`--recursive`, gemote automatically discovers git submodules and nested repos. Their remotes are stored under `[submodules."<path>"]`:

```toml
[remotes.origin]
url = "git@github.com:org/repo.git"

[submodules."libs/core".remotes.origin]
url = "git@github.com:org/core.git"

[submodules."libs/core".remotes.upstream]
url = "git@github.com:upstream/core.git"
```
