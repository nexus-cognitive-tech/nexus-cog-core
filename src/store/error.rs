//! Error types for `cog-store`.

use thiserror::Error;

/// Result alias for `cog-store`.
pub type StoreResult<T> = Result<T, StoreError>;

/// Errors produced by the store.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StoreError {
    /// I/O error.
    #[error("io error: {0}")]
    Io(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Deserialization error.
    #[error("deserialization error: {0}")]
    Deserialization(String),

    /// Lock acquisition failed.
    #[error("lock acquisition failed for `{0}`")]
    LockFailed(String),

    /// Migration failed.
    #[error("migration `{from}` -> `{to}` failed: {message}")]
    MigrationFailed {
        /// From version.
        from: String,
        /// To version.
        to: String,
        /// Error message.
        message: String,
    },

    /// File not found.
    #[error("file not found: {0}")]
    NotFound(String),
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}
