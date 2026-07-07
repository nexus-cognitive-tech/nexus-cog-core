//! Settings structures loaded from TOML.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::env::{env_var, env_var_bool, keys};
use super::error::{ConfigError, ConfigResult};
use super::paths::NexusPaths;

/// Top-level Nexus configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NexusConfig {
    /// Logging configuration.
    #[serde(default)]
    pub log: LogConfig,
    /// Provider configurations.
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    /// Server configuration.
    #[serde(default)]
    pub server: ServerConfig,
    /// Storage configuration.
    #[serde(default)]
    pub storage: StorageConfig,
}

/// Logging configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogConfig {
    /// Log level.
    pub level: LogLevel,
    /// Whether to log to a file in addition to stderr.
    pub to_file: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            to_file: false,
        }
    }
}

/// Logging verbosity.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Trace-level.
    Trace,
    /// Debug-level.
    Debug,
    /// Info-level (default).
    #[default]
    Info,
    /// Warning-level.
    Warn,
    /// Error-level.
    Error,
}

impl LogLevel {
    /// Parse from a string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "trace" => Some(Self::Trace),
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" | "warning" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    /// Returns the string used by `tracing`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Configuration for an LLM provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name (e.g. `"default"`, `"openai"`, `"anthropic"`).
    pub name: String,
    /// Base URL for the OpenAI-compatible API.
    pub base_url: String,
    /// API key.
    pub api_key: String,
    /// Model name.
    pub model: String,
    /// Maximum tokens per request.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Sampling temperature.
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_max_tokens() -> u32 {
    4096
}
fn default_temperature() -> f32 {
    0.7
}

/// Server configuration (MCP).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Maximum tool-call rounds per agent run.
    pub max_rounds: usize,
    /// Whether to enable the cognitive tools.
    pub enable_cognitive: bool,
    /// Whether to enable provenance tracking.
    pub enable_provenance: bool,
    /// Whether to enable causal reasoning tools.
    pub enable_causal: bool,
    /// Whether to enable antifragile verification.
    pub enable_antifragile: bool,
    /// Whether to enable intent preservation.
    pub enable_intent: bool,
    /// Agent identifier used in provenance records.
    pub agent_id: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_rounds: 20,
            enable_cognitive: true,
            enable_provenance: true,
            enable_causal: true,
            enable_antifragile: true,
            enable_intent: true,
            agent_id: env_var(keys::AGENT_ID).unwrap_or_else(|| "nexus-cog".to_string()),
        }
    }
}

/// Storage configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Whether to persist state across sessions.
    pub persist: bool,
    /// Path to the SQLite palace database file.
    #[serde(default = "default_db_path")]
    pub db_path: String,
    /// Backend for palace persistence: "local" or "cloud".
    #[serde(default = "default_backend")]
    pub backend: String,
    /// Cloud API base URL (only used when backend = "cloud").
    #[serde(default)]
    pub cloud_url: String,
    /// Cloud API key (only used when backend = "cloud").
    #[serde(default)]
    pub cloud_api_key: String,
}

fn default_backend() -> String {
    "local".to_string()
}

fn default_db_path() -> String {
    crate::config::paths::NexusPaths::defaults()
        .data_dir
        .join("palace.db")
        .to_string_lossy()
        .into_owned()
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            persist: env_var_bool(keys::NO_PERSIST) != Some(true),
            db_path: default_db_path(),
            backend: default_backend(),
            cloud_url: String::new(),
            cloud_api_key: String::new(),
        }
    }
}

impl NexusConfig {
    /// Load from the default config path.
    pub fn load_default() -> ConfigResult<Self> {
        let paths = NexusPaths::defaults();
        if let Some(p) = env_var(keys::CONFIG_PATH) {
            return Self::load_from_file(std::path::Path::new(&p));
        }
        if paths.config_file.exists() {
            return Self::load_from_file(&paths.config_file);
        }
        Ok(Self::default())
    }

    /// Load from a specific TOML file.
    pub fn load_from_file(path: &Path) -> ConfigResult<Self> {
        let raw = std::fs::read_to_string(path)?;
        let mut cfg: NexusConfig = toml::from_str(&raw)?;
        cfg.apply_env_overrides();
        Ok(cfg)
    }

    /// Apply environment-variable overrides on top of the parsed config.
    pub fn apply_env_overrides(&mut self) {
        if let Some(level) = env_var(keys::LOG_LEVEL).and_then(|s| LogLevel::parse(&s)) {
            self.log.level = level;
        }
        if let Some(_provider) = env_var(keys::DEFAULT_PROVIDER) {
            // TODO: use to select default provider from providers list
        }
        if env_var_bool(keys::NO_PERSIST) == Some(true) {
            self.storage.persist = false;
        }
    }

    /// Save to a TOML file.
    pub fn save_to_file(&self, path: &Path) -> ConfigResult<()> {
        let body = toml::to_string_pretty(self).map_err(|e| ConfigError::Toml(e.to_string()))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, body)?;
        Ok(())
    }

    /// Save to the default config file path.
    pub fn save_default(&self) -> ConfigResult<()> {
        let paths = NexusPaths::defaults();
        self.save_to_file(&paths.config_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn log_level_default_is_info() {
        assert_eq!(LogLevel::default(), LogLevel::Info);
    }

    #[test]
    fn log_level_parses_strings() {
        assert_eq!(LogLevel::parse("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::parse("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::parse("nonsense"), None);
    }

    #[test]
    fn roundtrip_through_toml() {
        let cfg = NexusConfig::default();
        let s = toml::to_string(&cfg).unwrap();
        let cfg2: NexusConfig = toml::from_str(&s).unwrap();
        assert_eq!(cfg, cfg2);
    }

    #[test]
    fn save_and_load_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("c.toml");
        let mut cfg = NexusConfig::default();
        cfg.providers.push(ProviderConfig {
            name: "test".into(),
            base_url: "https://example.com/v1".into(),
            api_key: "sk-test".into(),
            model: "test-model".into(),
            max_tokens: 2048,
            temperature: 0.5,
        });
        cfg.save_to_file(&path).unwrap();
        let loaded = NexusConfig::load_from_file(&path).unwrap();
        assert_eq!(loaded.providers.len(), 1);
    }
}
