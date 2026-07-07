//! Semantic-diff types.

use serde::{Deserialize, Serialize};

/// Type of change detected in a semantic diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// A new function was added.
    FunctionAdded,
    /// An existing function was removed.
    FunctionRemoved,
    /// An existing function was modified.
    FunctionModified,
    /// A type signature changed.
    TypeChanged,
    /// Control flow changed.
    LogicChanged,
    /// Internal refactor with no observable behavior change.
    Refactor,
    /// A bug fix.
    Bug,
    /// A performance optimization.
    Optimization,
    /// Only documentation changed.
    DocumentationOnly,
    /// Only formatting / whitespace changed.
    FormattingOnly,
}

impl ChangeType {
    /// Returns `true` if this change type is purely cosmetic (no behavior change).
    #[must_use]
    pub const fn is_cosmetic(self) -> bool {
        matches!(self, Self::DocumentationOnly | Self::FormattingOnly)
    }

    /// Returns `true` if this change type implies a behavior change.
    #[must_use]
    pub const fn is_behavioral(self) -> bool {
        matches!(
            self,
            Self::LogicChanged | Self::Bug | Self::Optimization | Self::FunctionModified
        )
    }
}

/// Location of a code change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeLocation {
    /// File.
    pub file: String,
    /// 1-indexed start line.
    pub line_start: u32,
    /// 1-indexed end line.
    pub line_end: u32,
    /// Optional context (e.g. function signature).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl CodeLocation {
    /// Construct a new code location.
    #[must_use]
    pub fn new(file: impl Into<String>, line_start: u32, line_end: u32) -> Self {
        Self {
            file: file.into(),
            line_start,
            line_end,
            context: None,
        }
    }

    /// Attach context.
    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// A single change in a semantic diff.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeChange {
    /// Change type.
    pub change_type: ChangeType,
    /// Location of the change.
    pub location: CodeLocation,
    /// Original code (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// New code (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Semantic weight in `[0.0, 1.0]` — how impactful this change is.
    pub semantic_weight: f32,
    /// Symbol affected (function name, struct name, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

impl CodeChange {
    /// Construct a new code change.
    #[must_use]
    pub fn new(change_type: ChangeType, location: CodeLocation, semantic_weight: f32) -> Self {
        Self {
            change_type,
            location,
            before: None,
            after: None,
            semantic_weight: semantic_weight.clamp(0.0, 1.0),
            symbol: None,
        }
    }

    /// Attach before/after snippets.
    #[must_use]
    pub fn with_diff(mut self, before: impl Into<String>, after: impl Into<String>) -> Self {
        self.before = Some(before.into());
        self.after = Some(after.into());
        self
    }

    /// Attach the affected symbol name.
    #[must_use]
    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }
}

/// Aggregate impact score across a set of changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactScore {
    /// Complexity impact in `[0.0, 1.0]`.
    pub complexity: f32,
    /// Risk impact in `[0.0, 1.0]`.
    pub risk: f32,
    /// Performance impact in `[0.0, 1.0]`.
    pub performance: f32,
    /// Readability impact in `[0.0, 1.0]`.
    pub readability: f32,
    /// Testability impact in `[0.0, 1.0]`.
    pub testability: f32,
    /// Overall impact in `[0.0, 1.0]`.
    pub overall: f32,
}

impl ImpactScore {
    /// Returns `true` if the overall impact exceeds the threshold.
    #[must_use]
    pub fn is_significant(&self, threshold: f32) -> bool {
        self.overall >= threshold
    }
}

/// A semantic diff between two versions of a file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticDiff {
    /// File that changed.
    pub file: String,
    /// Individual changes.
    pub changes: Vec<CodeChange>,
    /// Aggregate impact.
    pub impact: ImpactScore,
    /// Stable identifier.
    pub id: String,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Optional commit / version label for the "before" version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before_label: Option<String>,
    /// Optional commit / version label for the "after" version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_label: Option<String>,
}

impl SemanticDiff {
    /// Returns only behavioral changes.
    pub fn behavioral_changes(&self) -> impl Iterator<Item = &CodeChange> {
        self.changes
            .iter()
            .filter(|c| c.change_type.is_behavioral())
    }

    /// Returns only cosmetic changes.
    pub fn cosmetic_changes(&self) -> impl Iterator<Item = &CodeChange> {
        self.changes.iter().filter(|c| c.change_type.is_cosmetic())
    }

    /// Returns the changes affecting a specific symbol.
    pub fn changes_to_symbol(&self, symbol: &str) -> impl Iterator<Item = &CodeChange> {
        self.changes
            .iter()
            .filter(move |c| c.symbol.as_deref() == Some(symbol))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosmetic_vs_behavioral() {
        assert!(ChangeType::DocumentationOnly.is_cosmetic());
        assert!(!ChangeType::LogicChanged.is_cosmetic());
        assert!(ChangeType::LogicChanged.is_behavioral());
        assert!(!ChangeType::Refactor.is_behavioral());
    }

    #[test]
    fn code_change_clamps_weight() {
        let c = CodeChange::new(ChangeType::Refactor, CodeLocation::new("f.rs", 1, 1), 5.0);
        assert_eq!(c.semantic_weight, 1.0);
    }

    #[test]
    fn behavioral_changes_filters() {
        let d = SemanticDiff {
            file: "x".into(),
            changes: vec![
                CodeChange::new(ChangeType::LogicChanged, CodeLocation::new("x", 1, 1), 0.8),
                CodeChange::new(
                    ChangeType::DocumentationOnly,
                    CodeLocation::new("x", 2, 2),
                    0.1,
                ),
                CodeChange::new(ChangeType::Bug, CodeLocation::new("x", 3, 3), 0.9),
            ],
            impact: ImpactScore {
                complexity: 0.5,
                risk: 0.5,
                performance: 0.5,
                readability: 0.5,
                testability: 0.5,
                overall: 0.5,
            },
            id: "x".into(),
            timestamp: chrono::Utc::now(),
            before_label: None,
            after_label: None,
        };
        let b: Vec<_> = d.behavioral_changes().collect();
        assert_eq!(b.len(), 2);
    }
}
