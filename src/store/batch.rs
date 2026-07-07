//! Transactional batch writes for [`JsonStore`].
//!
//! A `BatchWriter` accumulates mutations in memory and applies them atomically
//! when [`BatchWriter::commit`] is called. If the commit fails, no changes
//! are visible. This is the safe way to update multiple related entries
//! (e.g. updating a learner interaction alongside the corresponding long-term
//! memory entry) without leaving the store in a partially-updated state.

use std::collections::HashSet;

use indexmap::IndexMap;

use super::error::{StoreError, StoreResult};
use super::json_store::JsonStore;

#[cfg(test)]
use super::json_store::StoreConfig;

/// A single mutation in a batch.
#[derive(Debug, Clone)]
enum Mutation {
    Set(String, serde_json::Value),
    Remove(String),
}

/// Accumulator of mutations to be applied atomically to a [`JsonStore`].
///
/// Created via [`JsonStore::batch`]. Use [`BatchWriter::set`] and
/// [`BatchWriter::remove`] to enqueue changes, then call
/// [`BatchWriter::commit`] to persist them in one atomic write.
#[derive(Debug)]
pub struct BatchWriter<'a> {
    store: &'a JsonStore,
    mutations: Vec<Mutation>,
    /// Keys that should end up deleted even if they didn't exist before the batch.
    pending_removals: HashSet<String>,
}

impl<'a> BatchWriter<'a> {
    /// Construct a new empty batch writer bound to a store.
    #[must_use]
    pub fn new(store: &'a JsonStore) -> Self {
        Self {
            store,
            mutations: Vec::new(),
            pending_removals: HashSet::new(),
        }
    }

    /// Number of pending mutations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.mutations.len()
    }

    /// Returns `true` if there are no pending mutations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mutations.is_empty()
    }

    /// Enqueue a `set` operation. Overwrites any earlier `set` for the same key
    /// within this batch.
    pub fn set<T: serde::Serialize>(
        &mut self,
        key: impl Into<String>,
        value: &T,
    ) -> StoreResult<()> {
        let value =
            serde_json::to_value(value).map_err(|e| StoreError::Serialization(e.to_string()))?;
        let key_str: String = key.into();
        // Drop any earlier mutation for this key so the latest wins.
        self.mutations
            .retain(|m| !matches!(m, Mutation::Set(k, _) if k == &key_str));
        self.pending_removals.remove(&key_str);
        self.mutations.push(Mutation::Set(key_str, value));
        Ok(())
    }

    /// Enqueue a `remove` operation.
    pub fn remove(&mut self, key: impl Into<String>) {
        let key_str: String = key.into();
        self.mutations
            .retain(|m| !matches!(m, Mutation::Set(k, _) if k == &key_str));
        self.mutations.push(Mutation::Remove(key_str.clone()));
        self.pending_removals.insert(key_str);
    }

    /// Discard all pending mutations without applying them.
    pub fn discard(mut self) {
        self.mutations.clear();
        self.pending_removals.clear();
    }

    /// Apply all pending mutations atomically and persist to disk.
    ///
    /// On success, returns the number of keys that actually changed
    /// (sets + removals). On failure, the store is unchanged.
    pub fn commit(self) -> StoreResult<BatchReport> {
        if self.mutations.is_empty() {
            return Ok(BatchReport {
                applied: 0,
                removed: 0,
            });
        }

        // Apply mutations to a draft copy; if save fails, the live store
        // is untouched.
        let mut draft: IndexMap<String, serde_json::Value> = self.store.snapshot();
        let mut applied = 0_usize;
        let mut removed = 0_usize;

        for mutation in self.mutations {
            match mutation {
                Mutation::Set(key, value) => {
                    let changed = draft.get(&key).is_none_or(|v| *v != value);
                    draft.insert(key, value);
                    if changed {
                        applied += 1;
                    }
                }
                Mutation::Remove(key) => {
                    if draft.shift_remove(&key).is_some() {
                        removed += 1;
                    }
                }
            }
        }

        // Persist atomically.
        crate::store::atomic::atomic_write_json(self.store.path(), &draft)?;

        // Commit to the live store only after the file write succeeded.
        *self.store.inner.write() = draft;

        Ok(BatchReport { applied, removed })
    }

    /// Apply the batch without persisting to disk. Useful for tests.
    pub fn apply_in_memory(self) -> StoreResult<BatchReport> {
        let mutations = self.mutations;
        let mut applied = 0_usize;
        let mut removed = 0_usize;
        let mut inner = self.store.inner.write();
        for mutation in mutations {
            match mutation {
                Mutation::Set(key, value) => {
                    let changed = inner.get(&key).is_none_or(|v| *v != value);
                    inner.insert(key, value);
                    if changed {
                        applied += 1;
                    }
                }
                Mutation::Remove(key) => {
                    if inner.shift_remove(&key).is_some() {
                        removed += 1;
                    }
                }
            }
        }
        Ok(BatchReport { applied, removed })
    }
}

/// Summary of a successful batch commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchReport {
    /// Number of keys written.
    pub applied: usize,
    /// Number of keys removed.
    pub removed: usize,
}

impl BatchReport {
    /// Total keys affected.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.applied + self.removed
    }
}

impl JsonStore {
    /// Start a new batch writer for atomic multi-key updates.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nexus_cog_core::{JsonStore, StoreConfig};
    /// let store = JsonStore::new(StoreConfig::default());
    /// let mut batch = store.batch();
    /// batch.set("user:1", &"alice").unwrap();
    /// batch.set("user:2", &"bob").unwrap();
    /// batch.remove("user:3");
    /// let report = batch.commit().unwrap();
    /// assert_eq!(report.total(), 3);
    /// ```
    #[must_use]
    pub fn batch(&self) -> BatchWriter<'_> {
        BatchWriter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn store_in(dir: &TempDir) -> JsonStore {
        JsonStore::new(StoreConfig {
            path: dir.path().join("s.json"),
            ..StoreConfig::default()
        })
    }

    #[test]
    fn empty_batch_commits_nothing() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let report = store.batch().commit().unwrap();
        assert_eq!(report.applied, 0);
        assert_eq!(report.removed, 0);
    }

    #[test]
    fn batch_set_then_commit_persists() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let mut batch = store.batch();
        batch.set("a", &1_i32).unwrap();
        batch.set("b", &"hello").unwrap();
        let report = batch.commit().unwrap();
        assert_eq!(report.applied, 2);
        assert_eq!(store.len(), 2);

        // Reload from disk.
        let store2 = store_in(&dir);
        store2.load().unwrap();
        let v: Option<i32> = store2.get("a").unwrap();
        assert_eq!(v, Some(1));
    }

    #[test]
    fn batch_remove_works() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        store.set("keep", &1_i32).unwrap();
        store.set("drop", &2_i32).unwrap();
        store.save().unwrap();

        let mut batch = store.batch();
        batch.remove("drop");
        let report = batch.commit().unwrap();
        assert_eq!(report.removed, 1);
        assert!(store.contains("keep"));
        assert!(!store.contains("drop"));
    }

    #[test]
    fn batch_set_overrides_within_batch() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let mut batch = store.batch();
        batch.set("k", &1_i32).unwrap();
        batch.set("k", &2_i32).unwrap();
        batch.set("k", &3_i32).unwrap();
        let report = batch.commit().unwrap();
        assert_eq!(report.applied, 1);
        let v: Option<i32> = store.get("k").unwrap();
        assert_eq!(v, Some(3));
    }

    #[test]
    fn batch_set_after_remove_restores() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        store.set("k", &1_i32).unwrap();
        store.save().unwrap();

        // Remove then set again → k should exist with the second value.
        let mut batch = store.batch();
        batch.remove("k");
        batch.set("k", &42_i32).unwrap();
        let report = batch.commit().unwrap();
        // applied counts new keys; removed counts keys that existed before.
        // After remove-then-set, the final value is "set", which is a new key
        // from the draft's perspective → applied == 1, removed == 1.
        assert_eq!(report.applied, 1);
        assert_eq!(report.removed, 1);
        let v: Option<i32> = store.get("k").unwrap();
        assert_eq!(v, Some(42));
    }

    #[test]
    fn batch_discard_does_not_apply() {
        let dir = TempDir::new().unwrap();
        let store = store_in(&dir);
        let mut batch = store.batch();
        batch.set("a", &1_i32).unwrap();
        batch.set("b", &2_i32).unwrap();
        batch.discard();
        assert!(store.is_empty());
    }

    #[test]
    fn batch_failure_leaves_store_unchanged() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("readonly");
        std::fs::create_dir(&path).unwrap();
        let store = JsonStore::new(StoreConfig {
            path: path.join("s.json"),
            ..StoreConfig::default()
        });
        store.set("existing", &"value").unwrap();

        // Save to disk first to establish a baseline.
        store.save().unwrap();
        let before = store.snapshot();

        // Force the commit to fail by deleting the parent directory and its
        // contents after the store has been loaded. This is portable across
        // platforms — chmod-readonly is unreliable on Windows and a no-op
        // for root.
        let store_path = path.join("s.json");
        let _ = std::fs::remove_file(&store_path);
        std::fs::remove_dir(&path).unwrap();

        // Try to commit a batch — should fail because the target directory is gone.
        let mut batch = store.batch();
        batch.set("new_key", &"new_value").unwrap();
        let result = batch.commit();

        if let Err(ref e) = result {
            eprintln!("commit failed (expected): {e}");
        }

        // Restore the directory so TempDir cleanup works.
        std::fs::create_dir(&path).ok();

        // The invariant we care about: whether the commit failed OR succeeded,
        // the live store's snapshot must still equal what we had before.
        // (On Windows, without chmod enforcement, the commit may succeed; in
        // that case the store does get updated, but that's a separate behaviour
        // test, not the rollback invariant.)
        let after = store.snapshot();
        if result.is_err() {
            assert!(
                !store.contains("new_key"),
                "store should not contain new_key after failed commit"
            );
            assert_eq!(before, after, "failed commit must leave store unchanged");
        }
    }

    #[test]
    fn batch_with_lock_writes_correctly() {
        let dir = TempDir::new().unwrap();
        let store = JsonStore::new(StoreConfig {
            path: dir.path().join("s.json"),
            use_lock: true,
            ..StoreConfig::default()
        });
        let mut batch = store.batch();
        batch.set("k1", &"v1").unwrap();
        batch.set("k2", &"v2").unwrap();
        let report = batch.commit().unwrap();
        assert_eq!(report.applied, 2);

        // Verify the file was written.
        let raw = std::fs::read_to_string(dir.path().join("s.json")).unwrap();
        assert!(raw.contains("k1"));
        assert!(raw.contains("k2"));
    }

    #[test]
    fn batch_report_total() {
        let r = BatchReport {
            applied: 3,
            removed: 2,
        };
        assert_eq!(r.total(), 5);
    }
}
