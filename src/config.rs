use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::GemoteError;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GemoteConfig {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub remotes: BTreeMap<String, RemoteConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub extra_remotes: ExtraRemotes,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtraRemotes {
    #[default]
    Ignore,
    Warn,
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_url: Option<String>,
}

pub fn load_config(path: &Path) -> Result<GemoteConfig, GemoteError> {
    if !path.exists() {
        return Err(GemoteError::ConfigNotFound(path.to_path_buf()));
    }
    let contents = std::fs::read_to_string(path)?;
    toml::from_str(&contents).map_err(GemoteError::ConfigParse)
}

pub fn serialize_config(config: &GemoteConfig) -> Result<String, GemoteError> {
    toml::to_string_pretty(config).map_err(GemoteError::ConfigSerialize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn default_extra_remotes_is_ignore() {
        assert_eq!(ExtraRemotes::default(), ExtraRemotes::Ignore);
    }

    #[test]
    fn extra_remotes_serde_roundtrip() {
        // Wrap in Settings since toml can't serialize a bare enum
        for variant in [
            ExtraRemotes::Ignore,
            ExtraRemotes::Warn,
            ExtraRemotes::Remove,
        ] {
            let settings = Settings {
                extra_remotes: variant.clone(),
            };
            let serialized = toml::to_string(&settings).unwrap();
            let deserialized: Settings = toml::from_str(&serialized).unwrap();
            assert_eq!(deserialized.extra_remotes, variant);
        }
    }

    #[test]
    fn load_config_success() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(
            f,
            r#"
[settings]
extra_remotes = "warn"

[remotes.origin]
url = "git@github.com:org/repo.git"
push_url = "https://github.com/org/repo.git"
"#
        )
        .unwrap();

        let cfg = load_config(f.path()).unwrap();
        assert_eq!(cfg.settings.extra_remotes, ExtraRemotes::Warn);
        assert_eq!(cfg.remotes.len(), 1);
        let origin = &cfg.remotes["origin"];
        assert_eq!(origin.url, "git@github.com:org/repo.git");
        assert_eq!(
            origin.push_url.as_deref(),
            Some("https://github.com/org/repo.git")
        );
    }

    #[test]
    fn load_config_file_not_found() {
        let result = load_config(Path::new("/nonexistent/.gemote"));
        assert!(matches!(result, Err(GemoteError::ConfigNotFound(_))));
    }

    #[test]
    fn load_config_invalid_toml() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, "[remotes\norigin = {{ url = }}").unwrap();

        let result = load_config(f.path());
        assert!(matches!(result, Err(GemoteError::ConfigParse(_))));
    }

    #[test]
    fn load_config_minimal() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(
            f,
            r#"
[remotes.origin]
url = "https://example.com/repo.git"
"#
        )
        .unwrap();

        let cfg = load_config(f.path()).unwrap();
        assert_eq!(cfg.settings.extra_remotes, ExtraRemotes::Ignore);
        assert_eq!(cfg.remotes["origin"].url, "https://example.com/repo.git");
        assert!(cfg.remotes["origin"].push_url.is_none());
    }

    #[test]
    fn load_config_multiple_remotes() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(
            f,
            r#"
[remotes.origin]
url = "https://github.com/a.git"

[remotes.upstream]
url = "https://github.com/b.git"

[remotes.mirror]
url = "https://gitlab.com/c.git"
"#
        )
        .unwrap();

        let cfg = load_config(f.path()).unwrap();
        assert_eq!(cfg.remotes.len(), 3);
        assert!(cfg.remotes.contains_key("origin"));
        assert!(cfg.remotes.contains_key("upstream"));
        assert!(cfg.remotes.contains_key("mirror"));
    }

    #[test]
    fn serialize_config_empty() {
        let cfg = GemoteConfig::default();
        let output = serialize_config(&cfg).unwrap();
        // Should be valid TOML that round-trips
        let _: GemoteConfig = toml::from_str(&output).unwrap();
    }

    #[test]
    fn serialize_config_with_remotes() {
        let mut cfg = GemoteConfig::default();
        cfg.remotes.insert(
            "origin".into(),
            RemoteConfig {
                url: "https://example.com/repo.git".into(),
                push_url: None,
            },
        );
        let output = serialize_config(&cfg).unwrap();
        assert!(output.contains("origin"));
        assert!(output.contains("https://example.com/repo.git"));
    }

    #[test]
    fn serialize_push_url_skipped_when_none() {
        let mut cfg = GemoteConfig::default();
        cfg.remotes.insert(
            "origin".into(),
            RemoteConfig {
                url: "https://example.com/repo.git".into(),
                push_url: None,
            },
        );
        let output = serialize_config(&cfg).unwrap();
        assert!(!output.contains("push_url"));
    }

    #[test]
    fn roundtrip() {
        let mut cfg = GemoteConfig::default();
        cfg.settings.extra_remotes = ExtraRemotes::Remove;
        cfg.remotes.insert(
            "origin".into(),
            RemoteConfig {
                url: "git@github.com:org/repo.git".into(),
                push_url: Some("https://github.com/org/repo.git".into()),
            },
        );
        cfg.remotes.insert(
            "upstream".into(),
            RemoteConfig {
                url: "git@github.com:upstream/repo.git".into(),
                push_url: None,
            },
        );

        let serialized = serialize_config(&cfg).unwrap();
        let deserialized: GemoteConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.settings.extra_remotes, ExtraRemotes::Remove);
        assert_eq!(deserialized.remotes.len(), 2);
        assert_eq!(
            deserialized.remotes["origin"].url,
            "git@github.com:org/repo.git"
        );
        assert_eq!(
            deserialized.remotes["origin"].push_url.as_deref(),
            Some("https://github.com/org/repo.git")
        );
        assert_eq!(
            deserialized.remotes["upstream"].url,
            "git@github.com:upstream/repo.git"
        );
        assert!(deserialized.remotes["upstream"].push_url.is_none());
    }
}
