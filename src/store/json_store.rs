//! JSON-backed key-value store with file locking.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use parking_lot::RwLock;
use serde::{Serialize, de::DeserializeOwned};

use super::atomic::atomic_write_json;
use super::error::{StoreError, StoreResult};
use super::lock::FileLock;

/// Configuration for the JSON store.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoreConfig {
    /// Path to the store file.
    pub path: PathBuf,
    /// Whether to pretty-print the JSON output.
    pub pretty: bool,
    /// Whether to acquire a file lock on each save.
    pub use_lock: bool,
}

/// JSON-backed key-value store.
#[derive(Debug)]
pub struct JsonStore {
    pub(crate) config: StoreConfig,
    pub(crate) inner: RwLock<IndexMap<String, serde_json::Value>>,
}

impl JsonStore {
    /// Construct a new store with the given config.
    #[must_use]
    pub fn new(config: StoreConfig) -> Self {
        Self {
            config,
            inner: RwLock::new(IndexMap::new()),
        }
    }

    /// Load from disk.
    pub fn load(&self) -> StoreResult<()> {
        let path = &self.config.path;
        if !path.exists() {
            return Ok(());
        }
        let raw = std::fs::read_to_string(path)?;
        let parsed: IndexMap<String, serde_json::Value> = if raw.trim().is_empty() {
            IndexMap::new()
        } else {
            serde_json::from_str(&raw).map_err(|e| StoreError::Deserialization(e.to_string()))?
        };
        *self.inner.write() = parsed;
        Ok(())
    }

    /// Save to disk.
    pub fn save(&self) -> StoreResult<()> {
        let snapshot = self.inner.read().clone();
        if self.config.use_lock {
            let lock_path = self.lock_path();
            let _lock = FileLock::acquire_exclusive(&lock_path)?;
            atomic_write_json(&self.config.path, &snapshot)?;
        } else {
            atomic_write_json(&self.config.path, &snapshot)?;
        }
        Ok(())
    }

    /// Returns the configured path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.config.path
    }

    /// Returns the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    /// Returns `true` if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }

    /// Check whether a key exists.
    #[must_use]
    pub fn contains(&self, key: &str) -> bool {
        self.inner.read().contains_key(key)
    }

    /// Get a value as a typed reference.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> StoreResult<Option<T>> {
        let guard = self.inner.read();
        match guard.get(key) {
            Some(v) => Ok(Some(
                serde_json::from_value(v.clone())
                    .map_err(|e| StoreError::Deserialization(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    /// Set a value.
    pub fn set<T: Serialize>(&self, key: impl Into<String>, value: &T) -> StoreResult<()> {
        let value =
            serde_json::to_value(value).map_err(|e| StoreError::Serialization(e.to_string()))?;
        self.inner.write().insert(key.into(), value);
        Ok(())
    }

    /// Remove a key.
    pub fn remove(&self, key: &str) -> StoreResult<bool> {
        Ok(self.inner.write().shift_remove(key).is_some())
    }

    /// Clear all entries.
    pub fn clear(&self) {
        self.inner.write().clear();
    }

    /// Snapshot all entries.
    #[must_use]
    pub fn snapshot(&self) -> IndexMap<String, serde_json::Value> {
        self.inner.read().clone()
    }

    fn lock_path(&self) -> PathBuf {
        let mut p = self.config.path.clone();
        let name = p
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "nexus-cog".into());
        p.set_file_name(format!(".{name}.lock"));
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn set_get_roundtrip() {
        let dir = TempDir::new().unwrap();
        let store = JsonStore::new(StoreConfig {
            path: dir.path().join("s.json"),
            ..StoreConfig::default()
        });
        store.set("k", &"hello".to_string()).unwrap();
        let v: Option<String> = store.get("k").unwrap();
        assert_eq!(v, Some("hello".to_string()));
    }

    #[test]
    fn save_and_load_persists() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("s.json");
        {
            let store = JsonStore::new(StoreConfig {
                path: path.clone(),
                use_lock: true,
                ..StoreConfig::default()
            });
            store.set("k", &42_i32).unwrap();
            store.save().unwrap();
        }
        let store = JsonStore::new(StoreConfig {
            path,
            use_lock: true,
            ..StoreConfig::default()
        });
        store.load().unwrap();
        let v: Option<i32> = store.get("k").unwrap();
        assert_eq!(v, Some(42));
    }

    #[test]
    fn remove_returns_bool() {
        let dir = TempDir::new().unwrap();
        let store = JsonStore::new(StoreConfig {
            path: dir.path().join("s.json"),
            ..StoreConfig::default()
        });
        store.set("k", &1_i32).unwrap();
        assert!(store.remove("k").unwrap());
        assert!(!store.remove("k").unwrap());
    }

    #[test]
    fn missing_file_loads_as_empty() {
        let dir = TempDir::new().unwrap();
        let store = JsonStore::new(StoreConfig {
            path: dir.path().join("nonexistent.json"),
            ..StoreConfig::default()
        });
        store.load().unwrap();
        assert_eq!(store.len(), 0);
    }
}
