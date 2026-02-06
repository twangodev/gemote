mod cli;
mod config;
mod error;
mod git;
mod sync;

use std::path::PathBuf;

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
        Commands::Sync { dry_run } => cmd_sync(&repo, &repo_root, cli.config, dry_run),
        Commands::Save { overwrite } => cmd_save(&repo, &repo_root, cli.config, overwrite),
    }
}

fn cmd_sync(
    repo: &git2::Repository,
    repo_root: &PathBuf,
    config_path: Option<PathBuf>,
    dry_run: bool,
) -> Result<()> {
    let config_file = config_path.unwrap_or_else(|| repo_root.join(".gemote"));
    let cfg = config::load_config(&config_file)
        .with_context(|| format!("Failed to load config from {}", config_file.display()))?;

    let local = git::list_remotes(repo).context("Failed to list local remotes")?;
    let actions = sync::compute_diff(&cfg, &local);

    if actions.is_empty() {
        println!("{}", "Already in sync. No changes needed.".green());
        return Ok(());
    }

    for action in &actions {
        println!("  {action}");
    }

    if dry_run {
        println!("\n{}", "(dry run — no changes applied)".dimmed());
    } else {
        sync::apply_actions(repo, &actions).context("Failed to apply sync actions")?;
        println!("\n{}", "Sync complete.".green().bold());
    }

    Ok(())
}

fn cmd_save(
    repo: &git2::Repository,
    repo_root: &PathBuf,
    config_path: Option<PathBuf>,
    overwrite: bool,
) -> Result<()> {
    let config_file = config_path.unwrap_or_else(|| repo_root.join(".gemote"));

    if config_file.exists() && !overwrite {
        anyhow::bail!(
            "{} already exists. Use --overwrite to replace it.",
            config_file.display()
        );
    }

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
