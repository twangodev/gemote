use std::collections::BTreeMap;
use std::fmt;

use colored::Colorize;

use crate::config::{ExtraRemotes, GemoteConfig};
use crate::error::GemoteError;
use crate::git::{self, RemoteInfo};

#[derive(Debug)]
pub enum SyncAction {
    Add {
        name: String,
        url: String,
        push_url: Option<String>,
    },
    UpdateUrl {
        name: String,
        old_url: String,
        new_url: String,
    },
    UpdatePushUrl {
        name: String,
        old: Option<String>,
        new: Option<String>,
    },
    Remove {
        name: String,
    },
}

impl fmt::Display for SyncAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncAction::Add { name, url, push_url } => {
                write!(f, "{} remote {} (url: {})", "add".green(), name.bold(), url)?;
                if let Some(pu) = push_url {
                    write!(f, " (push_url: {pu})")?;
                }
                Ok(())
            }
            SyncAction::UpdateUrl { name, old_url, new_url } => {
                write!(
                    f,
                    "{} remote {} url: {} -> {}",
                    "update".yellow(),
                    name.bold(),
                    old_url,
                    new_url
                )
            }
            SyncAction::UpdatePushUrl { name, old, new } => {
                write!(
                    f,
                    "{} remote {} push_url: {} -> {}",
                    "update".yellow(),
                    name.bold(),
                    old.as_deref().unwrap_or("(none)"),
                    new.as_deref().unwrap_or("(none)")
                )
            }
            SyncAction::Remove { name } => {
                write!(f, "{} remote {}", "remove".red(), name.bold())
            }
        }
    }
}

pub fn compute_diff(
    config: &GemoteConfig,
    local: &BTreeMap<String, RemoteInfo>,
) -> Vec<SyncAction> {
    let mut actions = Vec::new();

    // Check config remotes against local
    for (name, rc) in &config.remotes {
        match local.get(name) {
            None => {
                actions.push(SyncAction::Add {
                    name: name.clone(),
                    url: rc.url.clone(),
                    push_url: rc.push_url.clone(),
                });
            }
            Some(local_remote) => {
                if local_remote.url != rc.url {
                    actions.push(SyncAction::UpdateUrl {
                        name: name.clone(),
                        old_url: local_remote.url.clone(),
                        new_url: rc.url.clone(),
                    });
                }
                if local_remote.push_url != rc.push_url {
                    actions.push(SyncAction::UpdatePushUrl {
                        name: name.clone(),
                        old: local_remote.push_url.clone(),
                        new: rc.push_url.clone(),
                    });
                }
            }
        }
    }

    // Check local remotes not in config
    for name in local.keys() {
        if !config.remotes.contains_key(name) {
            match config.settings.extra_remotes {
                ExtraRemotes::Ignore => {}
                ExtraRemotes::Warn => {
                    eprintln!(
                        "{} remote '{}' exists locally but not in config",
                        "warning:".yellow().bold(),
                        name
                    );
                }
                ExtraRemotes::Remove => {
                    actions.push(SyncAction::Remove { name: name.clone() });
                }
            }
        }
    }

    actions
}

pub fn apply_actions(
    repo: &git2::Repository,
    actions: &[SyncAction],
) -> Result<(), GemoteError> {
    for action in actions {
        match action {
            SyncAction::Add { name, url, push_url } => {
                git::add_remote(repo, name, url, push_url.as_deref())?;
            }
            SyncAction::UpdateUrl { name, new_url, .. } => {
                git::update_remote_url(repo, name, new_url)?;
            }
            SyncAction::UpdatePushUrl { name, new, .. } => {
                git::update_remote_push_url(repo, name, new.as_deref())?;
            }
            SyncAction::Remove { name } => {
                git::remove_remote(repo, name)?;
            }
        }
    }
    Ok(())
}
