//! Configuration management: settings, XDG paths, environment variables.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod env;
pub mod error;
pub mod paths;
pub mod settings;

pub use error::{ConfigError, ConfigResult};
pub use paths::NexusPaths;
pub use settings::{LogLevel, NexusConfig, ProviderConfig, ServerConfig, StorageConfig};
