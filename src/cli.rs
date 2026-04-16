use std::path::PathBuf;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{ArgAction, Args, Parser, Subcommand};
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
        #[command(flatten)]
        recursive: RecursiveFlag,
    },
    /// Save current local remotes into .gemote
    Save {
        /// Overwrite existing .gemote file
        #[arg(long, short = 'f')]
        force: bool,
        #[command(flatten)]
        recursive: RecursiveFlag,
    },
    /// Generate shell completions
    Completions {
        /// The shell to generate completions for (bash, zsh, fish, powershell, elvish)
        shell: Shell,
    },
}

#[derive(Args, Debug, Clone, Default)]
pub struct RecursiveFlag {
    /// Also process submodules and nested repos (overrides settings.recursive)
    #[arg(
        long,
        short = 'r',
        action = ArgAction::SetTrue,
        overrides_with = "no_recursive",
    )]
    recursive: bool,
    /// Disable recursion for this invocation (overrides settings.recursive)
    #[arg(
        long = "no-recursive",
        action = ArgAction::SetTrue,
        overrides_with = "recursive",
    )]
    no_recursive: bool,
}

impl RecursiveFlag {
    pub fn resolve(&self) -> Option<bool> {
        if self.recursive {
            Some(true)
        } else if self.no_recursive {
            Some(false)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    fn sync_dry_run(cli: &Cli) -> bool {
        match &cli.command {
            Commands::Sync { dry_run, .. } => *dry_run,
            _ => panic!("expected sync"),
        }
    }

    fn save_force(cli: &Cli) -> bool {
        match &cli.command {
            Commands::Save { force, .. } => *force,
            _ => panic!("expected save"),
        }
    }

    fn sync_recursive(cli: &Cli) -> Option<bool> {
        match &cli.command {
            Commands::Sync { recursive, .. } => recursive.resolve(),
            _ => panic!("expected sync"),
        }
    }

    fn save_recursive(cli: &Cli) -> Option<bool> {
        match &cli.command {
            Commands::Save { recursive, .. } => recursive.resolve(),
            _ => panic!("expected save"),
        }
    }

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_sync() {
        let cli = Cli::try_parse_from(["gemote", "sync"]).unwrap();
        assert!(!sync_dry_run(&cli));
        assert_eq!(sync_recursive(&cli), None);
    }

    #[test]
    fn parse_sync_dry_run() {
        let cli = Cli::try_parse_from(["gemote", "sync", "--dry-run"]).unwrap();
        assert!(sync_dry_run(&cli));
        assert_eq!(sync_recursive(&cli), None);
    }

    #[test]
    fn parse_sync_recursive() {
        let cli = Cli::try_parse_from(["gemote", "sync", "--recursive"]).unwrap();
        assert_eq!(sync_recursive(&cli), Some(true));
    }

    #[test]
    fn parse_sync_recursive_short() {
        let cli = Cli::try_parse_from(["gemote", "sync", "-r"]).unwrap();
        assert_eq!(sync_recursive(&cli), Some(true));
    }

    #[test]
    fn parse_sync_no_recursive() {
        let cli = Cli::try_parse_from(["gemote", "sync", "--no-recursive"]).unwrap();
        assert_eq!(sync_recursive(&cli), Some(false));
    }

    #[test]
    fn parse_sync_last_flag_wins() {
        let cli = Cli::try_parse_from(["gemote", "sync", "--recursive", "--no-recursive"]).unwrap();
        assert_eq!(sync_recursive(&cli), Some(false));
    }

    #[test]
    fn parse_save() {
        let cli = Cli::try_parse_from(["gemote", "save"]).unwrap();
        assert!(!save_force(&cli));
        assert_eq!(save_recursive(&cli), None);
    }

    #[test]
    fn parse_save_force() {
        let cli = Cli::try_parse_from(["gemote", "save", "--force"]).unwrap();
        assert!(save_force(&cli));
        assert_eq!(save_recursive(&cli), None);
    }

    #[test]
    fn parse_save_force_short() {
        let cli = Cli::try_parse_from(["gemote", "save", "-f"]).unwrap();
        assert!(save_force(&cli));
        assert_eq!(save_recursive(&cli), None);
    }

    #[test]
    fn parse_save_recursive() {
        let cli = Cli::try_parse_from(["gemote", "save", "--recursive"]).unwrap();
        assert_eq!(save_recursive(&cli), Some(true));
    }

    #[test]
    fn parse_save_recursive_short() {
        let cli = Cli::try_parse_from(["gemote", "save", "-r"]).unwrap();
        assert_eq!(save_recursive(&cli), Some(true));
    }

    #[test]
    fn parse_save_no_recursive() {
        let cli = Cli::try_parse_from(["gemote", "save", "--no-recursive"]).unwrap();
        assert_eq!(save_recursive(&cli), Some(false));
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
