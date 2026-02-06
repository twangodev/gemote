use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gemote", about = "Declarative git remote management")]
pub struct Cli {
    /// Path to the .gemote config file
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Path to the git repository
    #[arg(long, global = true)]
    pub repo: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Sync local remotes to match the .gemote config
    Sync {
        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },
    /// Save current local remotes into .gemote
    Save {
        /// Overwrite existing .gemote file
        #[arg(long)]
        overwrite: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_sync() {
        let cli = Cli::try_parse_from(["gemote", "sync"]).unwrap();
        assert!(matches!(cli.command, Commands::Sync { dry_run: false }));
    }

    #[test]
    fn parse_sync_dry_run() {
        let cli = Cli::try_parse_from(["gemote", "sync", "--dry-run"]).unwrap();
        assert!(matches!(cli.command, Commands::Sync { dry_run: true }));
    }

    #[test]
    fn parse_save() {
        let cli = Cli::try_parse_from(["gemote", "save"]).unwrap();
        assert!(matches!(cli.command, Commands::Save { overwrite: false }));
    }

    #[test]
    fn parse_save_overwrite() {
        let cli = Cli::try_parse_from(["gemote", "save", "--overwrite"]).unwrap();
        assert!(matches!(cli.command, Commands::Save { overwrite: true }));
    }

    #[test]
    fn parse_global_flags() {
        let cli =
            Cli::try_parse_from(["gemote", "--config", "/tmp/cfg", "--repo", "/tmp/repo", "sync"])
                .unwrap();
        assert_eq!(cli.config.unwrap(), PathBuf::from("/tmp/cfg"));
        assert_eq!(cli.repo.unwrap(), PathBuf::from("/tmp/repo"));
    }
}
