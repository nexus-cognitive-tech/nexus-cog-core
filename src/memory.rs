//! Long-term-memory types.

use serde::{Deserialize, Serialize};

/// Category of long-term-memory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCategory {
    /// An architectural or implementation decision.
    Decision,
    /// A reusable code or design pattern.
    Pattern,
    /// An error to remember (and avoid).
    Error,
    /// A general learning.
    Learning,
    /// A user-stated preference.
    Preference,
    /// Project-specific context.
    Context,
    /// A fact about the codebase.
    Fact,
    /// An external reference (URL, doc).
    Reference,
}

impl MemoryCategory {
    /// Stable identifier.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Decision => "decision",
            Self::Pattern => "pattern",
            Self::Error => "error",
            Self::Learning => "learning",
            Self::Preference => "preference",
            Self::Context => "context",
            Self::Fact => "fact",
            Self::Reference => "reference",
        }
    }

    /// Returns all categories.
    #[must_use]
    pub const fn all() -> [MemoryCategory; 8] {
        [
            Self::Decision,
            Self::Pattern,
            Self::Error,
            Self::Learning,
            Self::Preference,
            Self::Context,
            Self::Fact,
            Self::Reference,
        ]
    }
}

/// A single long-term-memory entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Stable key (unique within the store).
    pub key: String,
    /// Stored value.
    pub value: String,
    /// Category.
    pub category: MemoryCategory,
    /// Importance in `[0.0, 1.0]`. Higher means harder to evict.
    pub importance: f32,
    /// Number of times this entry has been recalled.
    pub access_count: u32,
    /// Unix timestamp (seconds) of last access.
    pub last_accessed: i64,
    /// Tags for search.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Stable identifier.
    pub id: String,
    /// Timestamp when the entry was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Optional source pointer (e.g. file:line of the original observation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl MemoryEntry {
    /// Construct a new memory entry.
    ///
    /// `last_accessed` is initialised to the current Unix timestamp so the
    /// entry is **not** eligible for TTL pruning on its first decay pass.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        value: impl Into<String>,
        category: MemoryCategory,
        importance: f32,
    ) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            category,
            importance: importance.clamp(0.0, 1.0),
            access_count: 0,
            last_accessed: chrono::Utc::now().timestamp(),
            tags: Vec::new(),
            id: uuid::Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now(),
            source: None,
        }
    }
}

/// Statistics aggregated from long-term memory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total entries.
    pub total_entries: usize,
    /// Total accesses across all entries.
    pub total_accesses: u64,
    /// Average importance.
    pub avg_importance: f32,
    /// Number of distinct categories used.
    pub categories: usize,
    /// Per-category counts.
    pub by_category: indexmap::IndexMap<String, usize>,
    /// Estimated memory footprint in bytes (best-effort).
    pub estimated_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_ids_are_stable() {
        assert_eq!(MemoryCategory::Decision.id(), "decision");
        assert_eq!(MemoryCategory::Learning.id(), "learning");
    }

    #[test]
    fn all_returns_eight() {
        assert_eq!(MemoryCategory::all().len(), 8);
    }

    #[test]
    fn entry_importance_is_clamped() {
        let e = MemoryEntry::new("k", "v", MemoryCategory::Learning, 2.0);
        assert_eq!(e.importance, 1.0);
        let e = MemoryEntry::new("k", "v", MemoryCategory::Learning, -1.0);
        assert_eq!(e.importance, 0.0);
    }
}
