//! Intent-preservation types.

use serde::{Deserialize, Serialize};

use super::common::{Confidence, Range, Severity};

/// Operator used in an invariant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvariantOperator {
    /// Equal to.
    Equal,
    /// Not equal to.
    NotEqual,
    /// Less than.
    Less,
    /// Less than or equal.
    LessOrEqual,
    /// Greater than.
    Greater,
    /// Greater than or equal.
    GreaterOrEqual,
    /// Contains a substring / element.
    Contains,
    /// Does not contain.
    NotContains,
    /// Has length equal to.
    LengthEqual,
    /// Has length in range.
    LengthInRange,
    /// Matches a regular expression.
    MatchesRegex,
    /// Implements a trait.
    Implements,
    /// Calls a function (side effect).
    Calls,
    /// Does not panic.
    DoesNotPanic,
    /// Returns a specific type.
    ReturnsType,
}

impl InvariantOperator {
    /// Stable identifier.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Equal => "eq",
            Self::NotEqual => "neq",
            Self::Less => "lt",
            Self::LessOrEqual => "lte",
            Self::Greater => "gt",
            Self::GreaterOrEqual => "gte",
            Self::Contains => "contains",
            Self::NotContains => "not_contains",
            Self::LengthEqual => "len_eq",
            Self::LengthInRange => "len_in_range",
            Self::MatchesRegex => "matches_regex",
            Self::Implements => "implements",
            Self::Calls => "calls",
            Self::DoesNotPanic => "does_not_panic",
            Self::ReturnsType => "returns_type",
        }
    }
}

/// An invariant that a module/function must preserve.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Invariant {
    /// Stable identifier.
    pub id: String,
    /// Human-readable description.
    pub description: String,
    /// Operator.
    pub op: InvariantOperator,
    /// Left-hand side (expression / symbol).
    pub lhs: String,
    /// Right-hand side (expression / value).
    pub rhs: String,
    /// Severity if the invariant is violated.
    pub severity: Severity,
    /// Whether the invariant currently holds.
    #[serde(default)]
    pub holds: bool,
    /// Source of the check (file:line of the assertion).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl Invariant {
    /// Construct a new invariant.
    #[must_use]
    pub fn new(
        description: impl Into<String>,
        op: InvariantOperator,
        lhs: impl Into<String>,
        rhs: impl Into<String>,
        severity: Severity,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.into(),
            op,
            lhs: lhs.into(),
            rhs: rhs.into(),
            severity,
            holds: true,
            source: None,
        }
    }
}

/// Declared intent for a module / function.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleIntent {
    /// Stable identifier.
    pub id: String,
    /// Module / function this intent applies to.
    pub module: String,
    /// What this module is supposed to do.
    pub purpose: String,
    /// All invariants this module must preserve.
    pub invariants: Vec<Invariant>,
    /// Tags for grouping.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Author / source.
    #[serde(default)]
    pub author: String,
    /// When the intent was declared.
    pub declared_at: chrono::DateTime<chrono::Utc>,
    /// When the intent was last verified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_verified_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ModuleIntent {
    /// Returns `true` if all invariants currently hold.
    #[must_use]
    pub fn is_preserved(&self) -> bool {
        self.invariants.iter().all(|i| i.holds)
    }

    /// Returns invariants that don't currently hold.
    pub fn violated(&self) -> impl Iterator<Item = &Invariant> {
        self.invariants.iter().filter(|i| !i.holds)
    }
}

/// Result of an intent-preservation check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentCheck {
    /// Stable identifier.
    pub id: String,
    /// Module checked.
    pub module: String,
    /// Intent Preservation Index (0–100).
    pub ipi: u32,
    /// All invariants, with `holds` updated.
    pub invariants: Vec<Invariant>,
    /// Drift details — what changed.
    #[serde(default)]
    pub drift: Vec<IntentDrift>,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Confidence in the check itself.
    pub confidence: Confidence,
}

impl IntentCheck {
    /// Returns `true` if intent is preserved (`ipi >= 80`).
    #[must_use]
    pub fn is_preserved(&self) -> bool {
        self.ipi >= 80
    }
}

/// A specific drift between declared intent and current code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentDrift {
    /// Stable identifier.
    pub id: String,
    /// Module where drift was detected.
    pub module: String,
    /// Invariant violated.
    pub invariant_id: String,
    /// Description of the drift.
    pub description: String,
    /// Severity.
    pub severity: Severity,
    /// Optional location in code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Range>,
    /// Suggested fix.
    pub suggested_fix: String,
}

/// Aggregate drift report across multiple modules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentDriftReport {
    /// Stable identifier.
    pub id: String,
    /// All drifts.
    pub drifts: Vec<IntentDrift>,
    /// Average IPI across all checked modules.
    pub average_ipi: f32,
    /// Lowest IPI across all checked modules.
    pub min_ipi: u32,
    /// Modules with significant drift (IPI < 60).
    pub modules_with_drift: Vec<String>,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_intent() -> ModuleIntent {
        let mut i = Invariant::new(
            "return value is non-negative",
            InvariantOperator::GreaterOrEqual,
            "result",
            "0",
            Severity::Error,
        );
        i.holds = false;
        ModuleIntent {
            id: "m1".into(),
            module: "math::sqrt".into(),
            purpose: "compute non-negative square root".into(),
            invariants: vec![i],
            tags: vec![],
            author: String::new(),
            declared_at: chrono::Utc::now(),
            last_verified_at: None,
        }
    }

    #[test]
    fn is_preserved_returns_false_if_any_invariant_violated() {
        let i = sample_intent();
        assert!(!i.is_preserved());
        assert_eq!(i.violated().count(), 1);
    }

    #[test]
    fn operator_ids_are_stable() {
        assert_eq!(InvariantOperator::GreaterOrEqual.id(), "gte");
        assert_eq!(InvariantOperator::DoesNotPanic.id(), "does_not_panic");
    }

    #[test]
    fn intent_check_threshold() {
        let c = IntentCheck {
            id: "x".into(),
            module: "x".into(),
            ipi: 79,
            invariants: vec![],
            drift: vec![],
            timestamp: chrono::Utc::now(),
            confidence: Confidence::new(1.0),
        };
        assert!(!c.is_preserved());
        let c2 = IntentCheck { ipi: 80, ..c };
        assert!(c2.is_preserved());
    }
}
