//! Error types for `cog-config`.

use thiserror::Error;

/// Result alias for `cog-config`.
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Errors produced by config operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// I/O error.
    #[error("io error: {0}")]
    Io(String),

    /// TOML parse error.
    #[error("toml parse error: {0}")]
    Toml(String),

    /// Missing required field.
    #[error("missing required field: {0}")]
    MissingField(String),

    /// Invalid value.
    #[error("invalid value for {field}: {message}")]
    InvalidValue {
        /// Field name.
        field: String,
        /// Diagnostic.
        message: String,
    },
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::Toml(e.to_string())
    }
}
