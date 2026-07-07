//! Schema migrations.

use std::collections::BTreeMap;

use super::error::{StoreError, StoreResult};

/// A schema migration from one version to the next.
pub trait Migration: Send + Sync {
    /// Source version (e.g. `"0.1.0"`).
    fn from(&self) -> &str;
    /// Target version (e.g. `"0.2.0"`).
    fn to(&self) -> &str;
    /// Apply the migration to the data, returning the new data.
    fn apply(&self, data: serde_json::Value) -> StoreResult<serde_json::Value>;
}

/// Engine that runs a sequence of migrations in order.
pub struct MigrationEngine {
    migrations: BTreeMap<String, Box<dyn Migration>>,
}

impl std::fmt::Debug for MigrationEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MigrationEngine")
            .field("migrations", &self.migrations.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl Default for MigrationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationEngine {
    /// Construct an empty engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            migrations: BTreeMap::new(),
        }
    }

    /// Register a migration.
    pub fn register(&mut self, migration: Box<dyn Migration>) {
        self.migrations
            .insert(migration.from().to_string(), migration);
    }

    /// Run all migrations from `current` version forward, mutating `data` in place.
    pub fn run(
        &self,
        current: &str,
        target: &str,
        mut data: serde_json::Value,
    ) -> StoreResult<serde_json::Value> {
        let path = self.find_path(current, target)?;
        for key in path {
            let m = self
                .migrations
                .get(&key)
                .ok_or_else(|| StoreError::MigrationFailed {
                    from: key.clone(),
                    to: String::new(),
                    message: "migration not registered".into(),
                })?;
            data = m.apply(data)?;
        }
        Ok(data)
    }

    /// List registered `from` versions in order.
    #[must_use]
    pub fn versions(&self) -> Vec<&str> {
        self.migrations.keys().map(|s| s.as_str()).collect()
    }

    fn find_path(&self, current: &str, target: &str) -> StoreResult<Vec<String>> {
        let mut path = Vec::new();
        let mut cursor = current.to_string();
        while cursor != target {
            let Some(m) = self.migrations.get(&cursor) else {
                return Err(StoreError::MigrationFailed {
                    from: cursor,
                    to: target.into(),
                    message: "no migration from this version".into(),
                });
            };
            let next = m.to().to_string();
            path.push(cursor.clone());
            cursor = next;
            if path.len() > 1000 {
                return Err(StoreError::MigrationFailed {
                    from: current.into(),
                    to: target.into(),
                    message: "migration chain too long".into(),
                });
            }
        }
        Ok(path)
    }
}

/// A no-op migration, useful as a placeholder.
pub struct NoopMigration {
    from: String,
    to: String,
}

impl NoopMigration {
    /// Construct a no-op migration.
    #[must_use]
    pub fn new(from: &'static str, to: &'static str) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
        }
    }
}

impl Migration for NoopMigration {
    fn from(&self) -> &str {
        &self.from
    }
    fn to(&self) -> &str {
        &self.to
    }
    fn apply(&self, data: serde_json::Value) -> StoreResult<serde_json::Value> {
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_engine_has_no_versions() {
        let e = MigrationEngine::new();
        assert!(e.versions().is_empty());
    }

    #[test]
    fn noop_migration_preserves_data() {
        let mut e = MigrationEngine::new();
        e.register(Box::new(NoopMigration::new("0.1.0", "0.2.0")));
        let result = e
            .run("0.1.0", "0.2.0", serde_json::json!({"x": 1}))
            .unwrap();
        assert_eq!(result, serde_json::json!({"x": 1}));
    }

    #[test]
    fn missing_migration_fails() {
        let e = MigrationEngine::new();
        let result = e.run("0.1.0", "0.2.0", serde_json::json!({}));
        assert!(result.is_err());
    }
}
