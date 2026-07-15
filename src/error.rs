//! Domain error types for rshelp.
//!
//! Network / parsing / cache failures are represented explicitly so the UI
//! layer can render a tailored, colorful panel instead of a generic message.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RsHelpError {
    #[error("could not find documentation for `{0}`")]
    NotFound(String),

    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("request to {url} timed out or failed: {source}")]
    Fetch {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("failed to parse documentation page: {0}")]
    Parse(String),

    #[error("no cached copy available for `{0}` (running with --offline)")]
    OfflineMiss(String),

    #[error("invalid item path: `{0}`")]
    InvalidPath(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, RsHelpError>;
