//! AetherDB unified error type.

use thiserror::Error;

/// The single error type used throughout AetherDB.
///
/// Each variant maps to a specific subsystem failure. Callers should
/// never swallow errors — propagate with `?` and let the boundary layer
/// decide whether to log or surface them.
#[derive(Debug, Error)]
pub enum AetherError {
    #[error("codec error: {0}")]
    Codec(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("catalog error: {0}")]
    Catalog(String),

    #[error("index error: {0}")]
    Index(String),

    #[error("query error: {0}")]
    Query(String),

    #[error("engine error: {0}")]
    Engine(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("object store error: {0}")]
    ObjectStore(#[from] object_store::Error),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("not found: {0}")]
    NotFound(String),
}

/// Convenience alias. All fallible functions in AetherDB return this.
pub type Result<T> = std::result::Result<T, AetherError>;
