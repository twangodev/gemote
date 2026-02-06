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
