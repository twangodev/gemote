mod cli;
mod config;
mod error;
mod git;
mod sync;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;

use cli::{Cli, Commands};
use config::{GemoteConfig, RemoteConfig};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let repo = git::open_repo(cli.repo.as_deref()).context("Could not open git repository")?;
    let repo_root = repo
        .workdir()
        .context("Repository has no working directory (bare repo)")?
        .to_path_buf();

    match cli.command {
        Commands::Sync {
            dry_run,
            recursive,
        } => cmd_sync(&repo, &repo_root, cli.config, dry_run, recursive),
        Commands::Save {
            overwrite,
            recursive,
        } => cmd_save(&repo, &repo_root, cli.config, overwrite, recursive),
    }
}

fn cmd_sync(
    repo: &git2::Repository,
    repo_root: &Path,
    config_path: Option<PathBuf>,
    dry_run: bool,
    recursive: bool,
) -> Result<()> {
    let config_file = config_path.unwrap_or_else(|| repo_root.join(".gemote"));
    let cfg = config::load_config(&config_file)
        .with_context(|| format!("Failed to load config from {}", config_file.display()))?;

    sync_one_repo(repo, &cfg, None, dry_run)?;

    if recursive {
        let sub_repos =
            git::collect_all_repos(repo, repo_root).context("Failed to discover sub-repos")?;

        // Warn about config sections with no matching repo
        let discovered_paths: std::collections::BTreeSet<String> =
            sub_repos.iter().map(|s| s.path.clone()).collect();
        for path in cfg.submodules.keys() {
            if !discovered_paths.contains(path) {
                eprintln!(
                    "{} config has submodule section '{}' but no matching repo found",
                    "warning:".yellow().bold(),
                    path
                );
            }
        }

        for sub in &sub_repos {
            if let Some(sub_cfg) = cfg.submodules.get(&sub.path) {
                println!("\n{} {}", "Submodule:".cyan().bold(), sub.path.bold());
                sync_one_repo(&sub.repo, sub_cfg, Some(&sub.path), dry_run)?;
                // Recurse into sub-submodules
                if !sub_cfg.submodules.is_empty()
                    && let Some(sub_root) = sub.repo.workdir()
                {
                    sync_submodules_recursive(
                        &sub.repo,
                        sub_root,
                        sub_cfg,
                        &sub.path,
                        dry_run,
                    )?;
                }
            } else {
                eprintln!(
                    "{} discovered repo '{}' has no config section (skipping)",
                    "warning:".yellow().bold(),
                    sub.path
                );
            }
        }
    }

    Ok(())
}

fn sync_submodules_recursive(
    parent_repo: &git2::Repository,
    parent_root: &Path,
    parent_cfg: &GemoteConfig,
    parent_path: &str,
    dry_run: bool,
) -> Result<()> {
    let sub_repos = git::collect_all_repos(parent_repo, parent_root)
        .context("Failed to discover sub-repos")?;
    for sub in &sub_repos {
        let full_path = format!("{}/{}", parent_path, sub.path);
        if let Some(sub_cfg) = parent_cfg.submodules.get(&sub.path) {
            println!("\n{} {}", "Submodule:".cyan().bold(), full_path.bold());
            sync_one_repo(&sub.repo, sub_cfg, Some(&full_path), dry_run)?;
            if !sub_cfg.submodules.is_empty()
                && let Some(sub_root) = sub.repo.workdir()
            {
                sync_submodules_recursive(&sub.repo, sub_root, sub_cfg, &full_path, dry_run)?;
            }
        } else {
            eprintln!(
                "{} discovered repo '{}' has no config section (skipping)",
                "warning:".yellow().bold(),
                full_path
            );
        }
    }
    Ok(())
}

fn sync_one_repo(
    repo: &git2::Repository,
    cfg: &GemoteConfig,
    label: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    let local = git::list_remotes(repo).context("Failed to list local remotes")?;
    let actions = sync::compute_diff(cfg, &local);

    if actions.is_empty() {
        let prefix = label
            .map(|l| format!("[{}] ", l))
            .unwrap_or_default();
        println!(
            "{}{}",
            prefix,
            "Already in sync. No changes needed.".green()
        );
        return Ok(());
    }

    for action in &actions {
        println!("  {action}");
    }

    if dry_run {
        println!("{}", "(dry run — no changes applied)".dimmed());
    } else {
        sync::apply_actions(repo, &actions).context("Failed to apply sync actions")?;
        let prefix = label
            .map(|l| format!("[{}] ", l))
            .unwrap_or_default();
        println!("{}{}", prefix, "Sync complete.".green().bold());
    }

    Ok(())
}

fn cmd_save(
    repo: &git2::Repository,
    repo_root: &Path,
    config_path: Option<PathBuf>,
    overwrite: bool,
    recursive: bool,
) -> Result<()> {
    let config_file = config_path.unwrap_or_else(|| repo_root.join(".gemote"));

    if config_file.exists() && !overwrite {
        anyhow::bail!(
            "{} already exists. Use --overwrite to replace it.",
            config_file.display()
        );
    }

    let mut cfg = save_one_repo(repo)?;

    if recursive {
        let sub_repos =
            git::collect_all_repos(repo, repo_root).context("Failed to discover sub-repos")?;
        for sub in &sub_repos {
            println!("{} {}", "Submodule:".cyan().bold(), sub.path.bold());
            let mut sub_cfg = save_one_repo(&sub.repo)?;
            // Recurse into sub-submodules
            if let Some(sub_root) = sub.repo.workdir() {
                save_submodules_recursive(&sub.repo, sub_root, &mut sub_cfg)?;
            }
            cfg.submodules.insert(sub.path.clone(), sub_cfg);
        }
    }

    let content = config::serialize_config(&cfg).context("Failed to serialize config")?;
    std::fs::write(&config_file, &content)
        .with_context(|| format!("Failed to write {}", config_file.display()))?;

    println!(
        "{} {}",
        "Saved remotes to".green(),
        config_file.display().to_string().bold()
    );

    Ok(())
}

fn save_submodules_recursive(
    parent_repo: &git2::Repository,
    parent_root: &Path,
    parent_cfg: &mut GemoteConfig,
) -> Result<()> {
    let sub_repos = git::collect_all_repos(parent_repo, parent_root)
        .context("Failed to discover sub-repos")?;
    for sub in &sub_repos {
        let mut sub_cfg = save_one_repo(&sub.repo)?;
        if let Some(sub_root) = sub.repo.workdir() {
            save_submodules_recursive(&sub.repo, sub_root, &mut sub_cfg)?;
        }
        parent_cfg.submodules.insert(sub.path.clone(), sub_cfg);
    }
    Ok(())
}

fn save_one_repo(repo: &git2::Repository) -> Result<GemoteConfig> {
    let local = git::list_remotes(repo).context("Failed to list local remotes")?;
    let mut cfg = GemoteConfig::default();
    for (name, info) in local {
        cfg.remotes.insert(
            name,
            RemoteConfig {
                url: info.url,
                push_url: info.push_url,
            },
        );
    }
    Ok(cfg)
}
