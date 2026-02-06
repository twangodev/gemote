# gemote

Declarative git remote management. Define your remotes in a `.gemote` file, commit it, and keep the whole team in sync.

## Install

```sh
cargo install --path .
```

## Usage

### `gemote save`

Write your current local remotes into a `.gemote` file:

```sh
gemote save
gemote save --overwrite  # replace existing .gemote
```

### `gemote sync`

Set your local remotes to match the `.gemote` config:

```sh
gemote sync
gemote sync --dry-run  # preview changes without applying
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