//! Verification result types.

use serde::{Deserialize, Serialize};

use super::common::{Position, Range};

/// Coarse complexity bucket for a piece of code. Used by the verifier to choose which
/// checks to run (an adaptive verification strategy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeComplexity {
    /// < 5 lines, no functions — basically trivial.
    Trivial,
    /// < 20 lines, no structs — simple.
    Simple,
    /// < 100 lines — moderate.
    Moderate,
    /// >= 100 lines or many structures — complex, full battery of checks.
    Complex,
}

impl CodeComplexity {
    /// Returns the minimum number of lines that triggers this bucket.
    #[must_use]
    pub const fn min_lines(self) -> usize {
        match self {
            Self::Trivial => 0,
            Self::Simple => 5,
            Self::Moderate => 20,
            Self::Complex => 100,
        }
    }

    /// Returns `true` if this complexity level triggers the safety check.
    #[must_use]
    pub const fn requires_safety_check(self) -> bool {
        matches!(self, Self::Complex)
    }
}

/// Static profile of a code snippet — pre-computed by the verifier to avoid re-parsing
/// inside each individual check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeProfile {
    /// Total line count.
    pub lines: usize,
    /// `true` if the code contains function definitions.
    pub has_functions: bool,
    /// `true` if the code contains struct or enum definitions.
    pub has_structs: bool,
    /// `true` if the code contains an `unsafe` block.
    pub has_unsafe: bool,
    /// `true` if the code performs file or stream I/O.
    pub has_io: bool,
    /// `true` if the code performs network operations.
    pub has_network: bool,
    /// `true` if the code reads from stdin or process arguments.
    pub has_user_input: bool,
    /// Complexity bucket derived from `lines` and structure.
    pub complexity: CodeComplexity,
    /// Detected programming language (best-effort).
    pub language: Option<super::common::Language>,
}

impl CodeProfile {
    /// Construct an empty profile (used in tests and as a default).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            lines: 0,
            has_functions: false,
            has_structs: false,
            has_unsafe: false,
            has_io: false,
            has_network: false,
            has_user_input: false,
            complexity: CodeComplexity::Trivial,
            language: None,
        }
    }
}

/// A single verification check result.
///
/// Each verifier check produces one of these. Checks are intentionally simple to make
/// it easy for the agent to consume the report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationCheck {
    /// Stable identifier of the check (e.g. `"syntax_structure"`, `"error_handling"`).
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Detailed explanation — successes get a one-liner, failures get a concrete
    /// description of what was found and where.
    pub details: String,
    /// Severity if the check fails. Informational when `passed = true`.
    #[serde(default)]
    pub severity: crate::common::Severity,
    /// Optional location in the source code where the issue was found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Range>,
}

impl VerificationCheck {
    /// Construct a passing check.
    #[must_use]
    pub fn pass(
        id: impl Into<String>,
        name: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            passed: true,
            details: details.into(),
            severity: crate::common::Severity::Info,
            location: None,
        }
    }

    /// Construct a failing check.
    #[must_use]
    pub fn fail(
        id: impl Into<String>,
        name: impl Into<String>,
        details: impl Into<String>,
        severity: crate::common::Severity,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            passed: false,
            details: details.into(),
            severity,
            location: None,
        }
    }

    /// Attach a location to this check.
    #[must_use]
    pub fn with_location(mut self, location: Range) -> Self {
        self.location = Some(location);
        self
    }

    /// Attach a position (single-line location) to this check.
    #[must_use]
    pub fn at(mut self, line: u32, column: u32) -> Self {
        self.location = Some(Range::line(line, column, column));
        self
    }
}

/// Aggregated result of verifying a code snippet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationResult {
    /// `true` if every applicable check passed.
    pub passed: bool,
    /// Score in `[0.0, 1.0]` — fraction of checks that passed.
    pub score: f32,
    /// All individual check results.
    pub checks: Vec<VerificationCheck>,
    /// Concatenated issues from failed checks. Order matches [`Self::checks`].
    pub issues: Vec<String>,
    /// Profile used for this verification.
    pub profile: CodeProfile,
    /// Stable identifier for this verification (UUID v4).
    pub id: String,
    /// Timestamp (RFC 3339) when verification ran.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Optional label supplied by the caller (e.g. file path).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl VerificationResult {
    /// Build a passing result for trivial inputs (no checks needed).
    #[must_use]
    pub fn trivial(label: Option<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            passed: true,
            score: 1.0,
            checks: Vec::new(),
            issues: Vec::new(),
            profile: CodeProfile::empty(),
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now,
            label,
        }
    }

    /// Returns the checks that failed.
    pub fn failed_checks(&self) -> impl Iterator<Item = &VerificationCheck> {
        self.checks.iter().filter(|c| !c.passed)
    }

    /// Returns the most severe failure (if any).
    #[must_use]
    pub fn worst_severity(&self) -> Option<crate::common::Severity> {
        self.failed_checks().map(|c| c.severity).max()
    }

    /// Returns issues at or above the given severity.
    #[must_use]
    pub fn issues_at_or_above(
        &self,
        threshold: crate::common::Severity,
    ) -> Vec<&VerificationCheck> {
        self.failed_checks()
            .filter(|c| c.severity >= threshold)
            .collect()
    }

    /// Returns the first failure at a specific position, if any.
    #[must_use]
    pub fn failure_at(&self, pos: Position) -> Option<&VerificationCheck> {
        self.failed_checks()
            .find(|c| c.location.is_some_and(|r| r.start.line == pos.line))
    }
}

/// Configuration for the verifier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifierConfig {
    /// Whether to run the safety check even on non-`unsafe` code.
    pub always_run_safety: bool,
    /// Whether to require documentation on all public functions.
    pub strict_docs: bool,
    /// Whether to allow `unwrap()` calls (used for tests / quick prototypes).
    pub allow_unwrap: bool,
    /// Minimum complexity at which to enable edge-case checks.
    pub edge_case_threshold: CodeComplexity,
    /// Maximum number of issues to surface (truncates the rest).
    pub max_issues: usize,
}

impl Default for VerifierConfig {
    fn default() -> Self {
        Self {
            always_run_safety: false,
            strict_docs: false,
            allow_unwrap: false,
            edge_case_threshold: CodeComplexity::Moderate,
            max_issues: 50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complexity_min_lines_is_monotonic() {
        assert!(CodeComplexity::Trivial.min_lines() < CodeComplexity::Simple.min_lines());
        assert!(CodeComplexity::Simple.min_lines() < CodeComplexity::Moderate.min_lines());
        assert!(CodeComplexity::Moderate.min_lines() < CodeComplexity::Complex.min_lines());
    }

    #[test]
    fn complex_requires_safety_check() {
        assert!(CodeComplexity::Complex.requires_safety_check());
        assert!(!CodeComplexity::Moderate.requires_safety_check());
    }

    #[test]
    fn trivial_result_is_passing() {
        let r = VerificationResult::trivial(Some("test".into()));
        assert!(r.passed);
        assert_eq!(r.score, 1.0);
        assert!(r.issues.is_empty());
    }

    #[test]
    fn failed_checks_filters_correctly() {
        let r = VerificationResult {
            passed: false,
            score: 0.5,
            checks: vec![
                VerificationCheck::pass("a", "a", "ok"),
                VerificationCheck::fail("b", "b", "bad", crate::common::Severity::Warning),
                VerificationCheck::pass("c", "c", "ok"),
                VerificationCheck::fail("d", "d", "very bad", crate::common::Severity::Critical),
            ],
            issues: vec!["bad".into(), "very bad".into()],
            profile: CodeProfile::empty(),
            id: "x".into(),
            timestamp: chrono::Utc::now(),
            label: None,
        };
        let failed: Vec<_> = r.failed_checks().collect();
        assert_eq!(failed.len(), 2);
        assert_eq!(failed[0].id, "b");
        assert_eq!(failed[1].id, "d");
    }

    #[test]
    fn worst_severity_returns_max() {
        let r = VerificationResult {
            passed: false,
            score: 0.0,
            checks: vec![
                VerificationCheck::fail("a", "a", "x", crate::common::Severity::Warning),
                VerificationCheck::fail("b", "b", "x", crate::common::Severity::Error),
                VerificationCheck::fail("c", "c", "x", crate::common::Severity::Critical),
            ],
            issues: vec![],
            profile: CodeProfile::empty(),
            id: "x".into(),
            timestamp: chrono::Utc::now(),
            label: None,
        };
        assert_eq!(r.worst_severity(), Some(crate::common::Severity::Critical));
    }

    #[test]
    fn check_with_location_stores_position() {
        let c = VerificationCheck::fail("x", "x", "x", crate::common::Severity::Error).at(10, 5);
        assert!(c.location.is_some());
        assert_eq!(c.location.unwrap().start.line, 10);
    }
}
