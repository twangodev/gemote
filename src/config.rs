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