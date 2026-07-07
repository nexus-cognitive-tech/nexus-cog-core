//! XDG-compliant paths.

use std::path::{Path, PathBuf};

use directories::ProjectDirs;

/// Resolved filesystem paths used by Nexus Cog.
#[derive(Debug, Clone)]
pub struct NexusPaths {
    /// Root config directory (`~/.config/nexus-cog/` on Linux).
    pub config_dir: PathBuf,
    /// Root data directory (`~/.local/share/nexus-cog/` on Linux).
    pub data_dir: PathBuf,
    /// Cache directory (`~/.cache/nexus-cog/` on Linux).
    pub cache_dir: PathBuf,
    /// Log directory (defaults to `<data_dir>/logs/`).
    pub log_dir: PathBuf,
    /// Path to the main config file.
    pub config_file: PathBuf,
    /// Path to the persistent state file.
    pub state_file: PathBuf,
}

impl NexusPaths {
    /// Compute default paths using the XDG / OS conventions.
    #[must_use]
    pub fn defaults() -> Self {
        if let Some(dirs) = ProjectDirs::from("com", "vitkuz573", "nexus-cog") {
            let config_dir = dirs.config_dir().to_path_buf();
            let data_dir = dirs.data_dir().to_path_buf();
            let cache_dir = dirs.cache_dir().to_path_buf();
            let log_dir = data_dir.join("logs");
            Self {
                config_file: config_dir.join("config.toml"),
                state_file: data_dir.join("state.json"),
                log_dir,
                config_dir,
                data_dir,
                cache_dir,
            }
        } else {
            Self::fallback()
        }
    }

    /// Fallback used when ProjectDirs is unavailable (e.g. unsupported OS).
    #[must_use]
    pub fn fallback() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = home.join(".config").join("nexus-cog");
        let data_dir = home.join(".local").join("share").join("nexus-cog");
        let cache_dir = home.join(".cache").join("nexus-cog");
        let log_dir = data_dir.join("logs");
        Self {
            config_file: config_dir.join("config.toml"),
            state_file: data_dir.join("state.json"),
            log_dir,
            config_dir,
            data_dir,
            cache_dir,
        }
    }

    /// Ensure all directories exist.
    pub fn ensure_all(&self) -> std::io::Result<()> {
        ensure(&self.config_dir)?;
        ensure(&self.data_dir)?;
        ensure(&self.cache_dir)?;
        ensure(&self.log_dir)?;
        Ok(())
    }

    /// Ensure only the data directory exists (for read-only tools).
    pub fn ensure_data(&self) -> std::io::Result<()> {
        ensure(&self.data_dir)?;
        Ok(())
    }

    /// Path to a file in the data directory.
    pub fn data_file(&self, name: &str) -> PathBuf {
        self.data_dir.join(name)
    }

    /// Path to a file in the config directory.
    pub fn config_subfile(&self, name: &str) -> PathBuf {
        self.config_dir.join(name)
    }

    /// Path to a file in the cache directory.
    pub fn cache_file(&self, name: &str) -> PathBuf {
        self.cache_dir.join(name)
    }
}

fn ensure(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_resolve() {
        let p = NexusPaths::defaults();
        assert!(p.config_file.ends_with("config.toml"));
        assert!(p.state_file.ends_with("state.json"));
    }

    #[test]
    fn fallback_works() {
        let p = NexusPaths::fallback();
        assert!(p.config_file.to_string_lossy().contains("nexus-cog"));
    }

    #[test]
    #[allow(unsafe_code)]
    fn ensure_all_creates_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: env var mutation in a single-threaded test.
        unsafe {
            std::env::set_var("HOME", tmp.path());
        }
        let p = NexusPaths::defaults();
        p.ensure_all().unwrap();
        assert!(
            p.config_dir.exists()
                || p.config_dir
                    .to_string_lossy()
                    .contains(tmp.path().to_string_lossy().as_ref())
        );
    }
}
