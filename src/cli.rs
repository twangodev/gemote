use std::path::PathBuf;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};
use clap_complete::Shell;

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Parser)]
#[command(name = "gemote", version, about = "Declarative git remote management.", styles = STYLES)]
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
        /// Also process submodules and nested repos
        #[arg(long, short = 'r')]
        recursive: bool,
    },
    /// Save current local remotes into .gemote
    Save {
        /// Overwrite existing .gemote file
        #[arg(long, short = 'f')]
        force: bool,
        /// Also save remotes for submodules and nested repos
        #[arg(long, short = 'r')]
        recursive: bool,
    },
    /// Generate shell completions
    Completions {
        /// The shell to generate completions for (bash, zsh, fish, powershell, elvish)
        shell: Shell,
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
        assert!(matches!(
            cli.command,
            Commands::Sync {
                dry_run: false,
                recursive: false
            }
        ));
    }

    #[test]
    fn parse_sync_dry_run() {
        let cli = Cli::try_parse_from(["gemote", "sync", "--dry-run"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Sync {
                dry_run: true,
                recursive: false
            }
        ));
    }

    #[test]
    fn parse_sync_recursive() {
        let cli = Cli::try_parse_from(["gemote", "sync", "--recursive"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Sync {
                dry_run: false,
                recursive: true
            }
        ));
    }

    #[test]
    fn parse_sync_recursive_short() {
        let cli = Cli::try_parse_from(["gemote", "sync", "-r"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Sync {
                dry_run: false,
                recursive: true
            }
        ));
    }

    #[test]
    fn parse_save() {
        let cli = Cli::try_parse_from(["gemote", "save"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Save {
                force: false,
                recursive: false
            }
        ));
    }

    #[test]
    fn parse_save_force() {
        let cli = Cli::try_parse_from(["gemote", "save", "--force"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Save {
                force: true,
                recursive: false
            }
        ));
    }

    #[test]
    fn parse_save_force_short() {
        let cli = Cli::try_parse_from(["gemote", "save", "-f"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Save {
                force: true,
                recursive: false
            }
        ));
    }

    #[test]
    fn parse_save_recursive() {
        let cli = Cli::try_parse_from(["gemote", "save", "--recursive"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Save {
                force: false,
                recursive: true
            }
        ));
    }

    #[test]
    fn parse_save_recursive_short() {
        let cli = Cli::try_parse_from(["gemote", "save", "-r"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Save {
                force: false,
                recursive: true
            }
        ));
    }

    #[test]
    fn parse_global_flags() {
        let cli = Cli::try_parse_from([
            "gemote",
            "--config",
            "/tmp/cfg",
            "--repo",
            "/tmp/repo",
            "sync",
        ])
        .unwrap();
        assert_eq!(cli.config.unwrap(), PathBuf::from("/tmp/cfg"));
        assert_eq!(cli.repo.unwrap(), PathBuf::from("/tmp/repo"));
    }

    #[test]
    fn parse_completions_bash() {
        let cli = Cli::try_parse_from(["gemote", "completions", "bash"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Completions { shell: Shell::Bash }
        ));
    }

    #[test]
    fn parse_completions_zsh() {
        let cli = Cli::try_parse_from(["gemote", "completions", "zsh"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Completions { shell: Shell::Zsh }
        ));
    }

    #[test]
    fn parse_completions_fish() {
        let cli = Cli::try_parse_from(["gemote", "completions", "fish"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Completions { shell: Shell::Fish }
        ));
    }

    #[test]
    fn parse_completions_invalid_shell() {
        assert!(Cli::try_parse_from(["gemote", "completions", "nushell"]).is_err());
    }
}
