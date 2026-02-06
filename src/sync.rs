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
            SyncAction::Add {
                name,
                url,
                push_url,
            } => {
                write!(f, "{} remote {} (url: {})", "add".green(), name.bold(), url)?;
                if let Some(pu) = push_url {
                    write!(f, " (push_url: {pu})")?;
                }
                Ok(())
            }
            SyncAction::UpdateUrl {
                name,
                old_url,
                new_url,
            } => {
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

pub fn apply_actions(repo: &git2::Repository, actions: &[SyncAction]) -> Result<(), GemoteError> {
    for action in actions {
        match action {
            SyncAction::Add {
                name,
                url,
                push_url,
            } => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{RemoteConfig, Settings};

    fn make_config(extra: ExtraRemotes, remotes: Vec<(&str, &str, Option<&str>)>) -> GemoteConfig {
        let mut cfg = GemoteConfig {
            settings: Settings {
                extra_remotes: extra,
            },
            remotes: BTreeMap::new(),
        };
        for (name, url, push_url) in remotes {
            cfg.remotes.insert(
                name.into(),
                RemoteConfig {
                    url: url.into(),
                    push_url: push_url.map(Into::into),
                },
            );
        }
        cfg
    }

    fn make_local(remotes: Vec<(&str, &str, Option<&str>)>) -> BTreeMap<String, RemoteInfo> {
        let mut map = BTreeMap::new();
        for (name, url, push_url) in remotes {
            map.insert(
                name.into(),
                RemoteInfo {
                    url: url.into(),
                    push_url: push_url.map(Into::into),
                },
            );
        }
        map
    }

    // --- compute_diff tests ---

    #[test]
    fn diff_empty_both() {
        let cfg = make_config(ExtraRemotes::Ignore, vec![]);
        let local = make_local(vec![]);
        assert!(compute_diff(&cfg, &local).is_empty());
    }

    #[test]
    fn diff_add_remote() {
        let cfg = make_config(
            ExtraRemotes::Ignore,
            vec![("origin", "https://example.com/repo.git", None)],
        );
        let local = make_local(vec![]);
        let actions = compute_diff(&cfg, &local);

        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            SyncAction::Add { name, url, push_url }
            if name == "origin" && url == "https://example.com/repo.git" && push_url.is_none()
        ));
    }

    #[test]
    fn diff_add_multiple() {
        let cfg = make_config(
            ExtraRemotes::Ignore,
            vec![
                ("origin", "https://a.com/repo.git", None),
                ("upstream", "https://b.com/repo.git", None),
            ],
        );
        let local = make_local(vec![]);
        let actions = compute_diff(&cfg, &local);
        assert_eq!(actions.len(), 2);
        assert!(actions.iter().all(|a| matches!(a, SyncAction::Add { .. })));
    }

    #[test]
    fn diff_no_changes() {
        let cfg = make_config(
            ExtraRemotes::Ignore,
            vec![("origin", "https://example.com/repo.git", None)],
        );
        let local = make_local(vec![("origin", "https://example.com/repo.git", None)]);
        assert!(compute_diff(&cfg, &local).is_empty());
    }

    #[test]
    fn diff_update_url() {
        let cfg = make_config(
            ExtraRemotes::Ignore,
            vec![("origin", "https://new.com/repo.git", None)],
        );
        let local = make_local(vec![("origin", "https://old.com/repo.git", None)]);
        let actions = compute_diff(&cfg, &local);

        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            SyncAction::UpdateUrl { name, old_url, new_url }
            if name == "origin" && old_url == "https://old.com/repo.git" && new_url == "https://new.com/repo.git"
        ));
    }

    #[test]
    fn diff_update_push_url_add() {
        let cfg = make_config(
            ExtraRemotes::Ignore,
            vec![(
                "origin",
                "https://example.com/repo.git",
                Some("git@example.com:repo.git"),
            )],
        );
        let local = make_local(vec![("origin", "https://example.com/repo.git", None)]);
        let actions = compute_diff(&cfg, &local);

        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            SyncAction::UpdatePushUrl { name, old, new }
            if name == "origin" && old.is_none() && new.as_deref() == Some("git@example.com:repo.git")
        ));
    }

    #[test]
    fn diff_update_push_url_remove() {
        let cfg = make_config(
            ExtraRemotes::Ignore,
            vec![("origin", "https://example.com/repo.git", None)],
        );
        let local = make_local(vec![(
            "origin",
            "https://example.com/repo.git",
            Some("git@example.com:repo.git"),
        )]);
        let actions = compute_diff(&cfg, &local);

        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            SyncAction::UpdatePushUrl { name, old, new }
            if name == "origin" && old.as_deref() == Some("git@example.com:repo.git") && new.is_none()
        ));
    }

    #[test]
    fn diff_update_push_url_change() {
        let cfg = make_config(
            ExtraRemotes::Ignore,
            vec![(
                "origin",
                "https://example.com/repo.git",
                Some("git@new.com:repo.git"),
            )],
        );
        let local = make_local(vec![(
            "origin",
            "https://example.com/repo.git",
            Some("git@old.com:repo.git"),
        )]);
        let actions = compute_diff(&cfg, &local);

        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            SyncAction::UpdatePushUrl { name, old, new }
            if name == "origin"
              && old.as_deref() == Some("git@old.com:repo.git")
              && new.as_deref() == Some("git@new.com:repo.git")
        ));
    }

    #[test]
    fn diff_extra_ignore() {
        let cfg = make_config(ExtraRemotes::Ignore, vec![]);
        let local = make_local(vec![("extra", "https://extra.com/repo.git", None)]);
        assert!(compute_diff(&cfg, &local).is_empty());
    }

    #[test]
    fn diff_extra_warn() {
        let cfg = make_config(ExtraRemotes::Warn, vec![]);
        let local = make_local(vec![("extra", "https://extra.com/repo.git", None)]);
        // Warn produces no actions (only stderr output)
        assert!(compute_diff(&cfg, &local).is_empty());
    }

    #[test]
    fn diff_extra_remove() {
        let cfg = make_config(ExtraRemotes::Remove, vec![]);
        let local = make_local(vec![("extra", "https://extra.com/repo.git", None)]);
        let actions = compute_diff(&cfg, &local);

        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            SyncAction::Remove { name } if name == "extra"
        ));
    }

    #[test]
    fn diff_complex() {
        let cfg = make_config(
            ExtraRemotes::Remove,
            vec![
                ("origin", "https://new-origin.com/repo.git", None),
                ("upstream", "https://upstream.com/repo.git", None),
            ],
        );
        let local = make_local(vec![
            ("origin", "https://old-origin.com/repo.git", None), // URL mismatch -> Update
            ("stale", "https://stale.com/repo.git", None),       // not in config -> Remove
                                                                 // upstream missing from local -> Add
        ]);
        let actions = compute_diff(&cfg, &local);

        assert_eq!(actions.len(), 3);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, SyncAction::UpdateUrl { name, .. } if name == "origin"))
        );
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, SyncAction::Add { name, .. } if name == "upstream"))
        );
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, SyncAction::Remove { name } if name == "stale"))
        );
    }

    // --- apply_actions tests ---

    fn test_repo() -> (tempfile::TempDir, git2::Repository) {
        let dir = tempfile::TempDir::new().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        (dir, repo)
    }

    #[test]
    fn apply_empty() {
        let (_dir, repo) = test_repo();
        apply_actions(&repo, &[]).unwrap();
        assert!(repo.remotes().unwrap().is_empty());
    }

    #[test]
    fn apply_add() {
        let (_dir, repo) = test_repo();
        let actions = vec![SyncAction::Add {
            name: "origin".into(),
            url: "https://example.com/repo.git".into(),
            push_url: None,
        }];
        apply_actions(&repo, &actions).unwrap();

        let remote = repo.find_remote("origin").unwrap();
        assert_eq!(remote.url().unwrap(), "https://example.com/repo.git");
    }

    #[test]
    fn apply_add_with_push_url() {
        let (_dir, repo) = test_repo();
        let actions = vec![SyncAction::Add {
            name: "origin".into(),
            url: "https://example.com/repo.git".into(),
            push_url: Some("git@example.com:repo.git".into()),
        }];
        apply_actions(&repo, &actions).unwrap();

        let remote = repo.find_remote("origin").unwrap();
        assert_eq!(remote.url().unwrap(), "https://example.com/repo.git");
        assert_eq!(remote.pushurl().unwrap(), "git@example.com:repo.git");
    }

    #[test]
    fn apply_update_url() {
        let (_dir, repo) = test_repo();
        repo.remote("origin", "https://old.com/repo.git").unwrap();

        let actions = vec![SyncAction::UpdateUrl {
            name: "origin".into(),
            old_url: "https://old.com/repo.git".into(),
            new_url: "https://new.com/repo.git".into(),
        }];
        apply_actions(&repo, &actions).unwrap();

        let remote = repo.find_remote("origin").unwrap();
        assert_eq!(remote.url().unwrap(), "https://new.com/repo.git");
    }

    #[test]
    fn apply_update_push_url() {
        let (_dir, repo) = test_repo();
        repo.remote("origin", "https://example.com/repo.git")
            .unwrap();

        let actions = vec![SyncAction::UpdatePushUrl {
            name: "origin".into(),
            old: None,
            new: Some("git@example.com:repo.git".into()),
        }];
        apply_actions(&repo, &actions).unwrap();

        let remote = repo.find_remote("origin").unwrap();
        assert_eq!(remote.pushurl().unwrap(), "git@example.com:repo.git");
    }

    #[test]
    fn apply_remove() {
        let (_dir, repo) = test_repo();
        repo.remote("origin", "https://example.com/repo.git")
            .unwrap();

        let actions = vec![SyncAction::Remove {
            name: "origin".into(),
        }];
        apply_actions(&repo, &actions).unwrap();

        assert!(repo.find_remote("origin").is_err());
    }
}
