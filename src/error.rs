use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum GemoteError {
    #[error("failed to discover git repository")]
    RepoNotFound(#[source] git2::Error),

    #[error("config file not found: {0}")]
    ConfigNotFound(PathBuf),

    #[error("failed to parse config: {0}")]
    ConfigParse(#[source] toml::de::Error),

    #[error("failed to serialize config")]
    ConfigSerialize(#[source] toml::ser::Error),

    #[error("git operation failed")]
    Git(#[from] git2::Error),

    #[error("IO error")]
    Io(#[from] std::io::Error),
}
