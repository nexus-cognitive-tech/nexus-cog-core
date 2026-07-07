//! Environment-variable overrides.

use std::env;

/// Read an environment variable, returning `None` if unset or empty.
#[must_use]
pub fn env_var(key: &str) -> Option<String> {
    env::var(key).ok().filter(|v| !v.is_empty())
}

/// Read an environment variable as a bool. Returns `None` if not parseable.
#[must_use]
pub fn env_var_bool(key: &str) -> Option<bool> {
    env_var(key).and_then(|v| match v.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    })
}

/// Read an environment variable as u32.
#[must_use]
pub fn env_var_u32(key: &str) -> Option<u32> {
    env_var(key).and_then(|v| v.parse().ok())
}

/// Read an environment variable as usize.
#[must_use]
pub fn env_var_usize(key: &str) -> Option<usize> {
    env_var(key).and_then(|v| v.parse().ok())
}

/// All environment variables used by Nexus Cog.
pub mod keys {
    /// Override config file path.
    pub const CONFIG_PATH: &str = "NEXUS_COG_CONFIG";
    /// Override data directory.
    pub const DATA_DIR: &str = "NEXUS_COG_DATA";
    /// Override log level (`trace`, `debug`, `info`, `warn`, `error`).
    pub const LOG_LEVEL: &str = "NEXUS_COG_LOG";
    /// Provider name to use by default.
    pub const DEFAULT_PROVIDER: &str = "NEXUS_COG_PROVIDER";
    /// Disable state persistence (1 = disable).
    pub const NO_PERSIST: &str = "NEXUS_COG_NO_PERSIST";
    /// Agent identifier used in provenance records.
    pub const AGENT_ID: &str = "NEXUS_COG_AGENT_ID";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(unsafe_code)]
    fn env_var_bool_parses() {
        // SAFETY: env var mutation in a single-threaded test.
        unsafe {
            std::env::set_var("NEXUS_TEST_BOOL", "true");
            assert_eq!(env_var_bool("NEXUS_TEST_BOOL"), Some(true));
            std::env::set_var("NEXUS_TEST_BOOL", "no");
            assert_eq!(env_var_bool("NEXUS_TEST_BOOL"), Some(false));
            std::env::remove_var("NEXUS_TEST_BOOL");
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn env_var_u32_parses() {
        // SAFETY: env var mutation in a single-threaded test.
        unsafe {
            std::env::set_var("NEXUS_TEST_U32", "42");
            assert_eq!(env_var_u32("NEXUS_TEST_U32"), Some(42));
            std::env::remove_var("NEXUS_TEST_U32");
        }
    }
}
