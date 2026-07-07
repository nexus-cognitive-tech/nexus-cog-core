//! Architecture analysis types.

use serde::{Deserialize, Serialize};

use super::common::{Range, Severity};

/// Type of architecture issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureIssueType {
    /// File with too many responsibilities.
    GodObject,
    /// Two modules import each other transitively.
    CircularDependency,
    /// Module depends on too many other modules.
    TightCoupling,
    /// Module has unrelated responsibilities.
    LowCohesion,
    /// Long inheritance chain.
    DeepInheritance,
    /// Method uses another class's data more than its own.
    FeatureEnvy,
    /// Function with too many lines.
    LongMethod,
    /// Struct / class with too many methods.
    LargeClass,
}

impl ArchitectureIssueType {
    /// Stable identifier.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::GodObject => "god_object",
            Self::CircularDependency => "circular_dependency",
            Self::TightCoupling => "tight_coupling",
            Self::LowCohesion => "low_cohesion",
            Self::DeepInheritance => "deep_inheritance",
            Self::FeatureEnvy => "feature_envy",
            Self::LongMethod => "long_method",
            Self::LargeClass => "large_class",
        }
    }
}

/// A single architecture issue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchitectureIssue {
    /// Type of issue.
    pub issue_type: ArchitectureIssueType,
    /// Severity.
    pub severity: Severity,
    /// File or module where the issue was found.
    pub location: String,
    /// Description of the issue.
    pub description: String,
    /// Suggested fix.
    pub fix: String,
    /// Optional range in the source code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
}

impl ArchitectureIssue {
    /// Construct a new architecture issue.
    #[must_use]
    pub fn new(
        issue_type: ArchitectureIssueType,
        severity: Severity,
        location: impl Into<String>,
        description: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self {
            issue_type,
            severity,
            location: location.into(),
            description: description.into(),
            fix: fix.into(),
            range: None,
        }
    }
}

/// A suggested design pattern that would improve the analyzed code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchitectureSuggestion {
    /// Pattern name (e.g. `"Builder"`, `"Repository"`).
    pub pattern: String,
    /// Description of when and why to apply the pattern.
    pub description: String,
    /// Confidence in `[0.0, 1.0]`.
    pub confidence: f32,
    /// Concrete examples of the pattern.
    pub examples: Vec<String>,
    /// Locations where this pattern would apply.
    #[serde(default)]
    pub target_locations: Vec<String>,
}

/// A design pattern detected in the analyzed code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectedPattern {
    /// Pattern name.
    pub name: String,
    /// Confidence in `[0.0, 1.0]`.
    pub confidence: f32,
    /// Files where the pattern was detected.
    pub locations: Vec<String>,
    /// Example snippets of the pattern in the codebase.
    #[serde(default)]
    pub examples: Vec<String>,
}

/// Aggregated architecture report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchitectureReport {
    /// Overall quality score in `[0.0, 1.0]`. Higher is better.
    pub score: f32,
    /// All detected issues.
    pub issues: Vec<ArchitectureIssue>,
    /// Suggested improvements.
    pub suggestions: Vec<ArchitectureSuggestion>,
    /// Detected patterns.
    pub patterns: Vec<DetectedPattern>,
    /// Statistics.
    pub stats: ArchitectureStats,
    /// Stable identifier.
    pub id: String,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ArchitectureReport {
    /// Returns the worst-severity issue.
    #[must_use]
    pub fn worst_issue(&self) -> Option<&ArchitectureIssue> {
        self.issues.iter().max_by_key(|i| i.severity)
    }

    /// Returns issues at or above the threshold.
    #[must_use]
    pub fn issues_at_or_above(&self, threshold: Severity) -> Vec<&ArchitectureIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity >= threshold)
            .collect()
    }
}

/// Statistics aggregated from an architecture report.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ArchitectureStats {
    /// Total issues detected.
    pub total_issues: usize,
    /// Total suggestions generated.
    pub total_suggestions: usize,
    /// Total patterns detected.
    pub total_patterns: usize,
    /// Breakdown of issues by type.
    pub by_type: indexmap::IndexMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_type_ids_are_stable() {
        assert_eq!(ArchitectureIssueType::GodObject.id(), "god_object");
        assert_eq!(ArchitectureIssueType::LongMethod.id(), "long_method");
    }

    #[test]
    fn worst_issue_returns_max_severity() {
        let report = ArchitectureReport {
            score: 0.5,
            issues: vec![
                ArchitectureIssue::new(
                    ArchitectureIssueType::LongMethod,
                    Severity::Warning,
                    "f.rs",
                    "long",
                    "split",
                ),
                ArchitectureIssue::new(
                    ArchitectureIssueType::GodObject,
                    Severity::Critical,
                    "g.rs",
                    "huge",
                    "split",
                ),
            ],
            suggestions: vec![],
            patterns: vec![],
            stats: ArchitectureStats::default(),
            id: "x".into(),
            timestamp: chrono::Utc::now(),
        };
        let worst = report.worst_issue().unwrap();
        assert_eq!(worst.severity, Severity::Critical);
        assert_eq!(worst.issue_type, ArchitectureIssueType::GodObject);
    }
}
