//! Antifragile-verification types.
//!
//! These types describe adversarial inputs, robustness reports, and edge-case exploration.

use serde::{Deserialize, Serialize};

use super::common::Severity;

/// Category of adversarial input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdversarialCategory {
    /// Empty inputs (empty string, empty array, …).
    Empty,
    /// Inputs at or near boundaries (min, max, off-by-one).
    Boundary,
    /// Inputs with special characters (unicode, RTL, …).
    SpecialCharacters,
    /// Inputs that are very large (memory pressure).
    Large,
    /// Inputs that are malformed.
    Malformed,
    /// Inputs that are repeated many times (loop stress).
    Repetition,
    /// Inputs containing null bytes / injection attempts.
    Injection,
    /// Inputs with unusual numeric values (NaN, infinity, negatives, …).
    NumericEdge,
    /// Type-confusion attempts.
    TypeConfusion,
    /// Concurrency stress (race conditions).
    Concurrency,
    /// Fuzz-style random inputs.
    Fuzz,
}

impl AdversarialCategory {
    /// Stable identifier.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Boundary => "boundary",
            Self::SpecialCharacters => "special_characters",
            Self::Large => "large",
            Self::Malformed => "malformed",
            Self::Repetition => "repetition",
            Self::Injection => "injection",
            Self::NumericEdge => "numeric_edge",
            Self::TypeConfusion => "type_confusion",
            Self::Concurrency => "concurrency",
            Self::Fuzz => "fuzz",
        }
    }

    /// Returns all categories.
    #[must_use]
    pub const fn all() -> [AdversarialCategory; 11] {
        [
            Self::Empty,
            Self::Boundary,
            Self::SpecialCharacters,
            Self::Large,
            Self::Malformed,
            Self::Repetition,
            Self::Injection,
            Self::NumericEdge,
            Self::TypeConfusion,
            Self::Concurrency,
            Self::Fuzz,
        ]
    }
}

/// A single adversarial input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdversarialInput {
    /// Stable identifier.
    pub id: String,
    /// Category.
    pub category: AdversarialCategory,
    /// Short description (e.g. `"empty string"`, `"u32::MAX"`).
    pub description: String,
    /// Serialized input value (string, JSON, etc.).
    pub value: String,
    /// Why this input is dangerous.
    pub rationale: String,
    /// Estimated likelihood of breaking the code in `[0.0, 1.0]`.
    pub break_likelihood: f32,
    /// Optional source / template.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl AdversarialInput {
    /// Construct a new adversarial input.
    #[must_use]
    pub fn new(
        category: AdversarialCategory,
        description: impl Into<String>,
        value: impl Into<String>,
        rationale: impl Into<String>,
        break_likelihood: f32,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            category,
            description: description.into(),
            value: value.into(),
            rationale: rationale.into(),
            break_likelihood: break_likelihood.clamp(0.0, 1.0),
            source: None,
        }
    }
}

/// Result of running an adversarial input against a function.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdversarialResult {
    /// The input that was tested.
    pub input: AdversarialInput,
    /// Whether the code handled it gracefully.
    pub handled: bool,
    /// Output (if any) when handled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Error message (if not handled).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Severity if not handled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
    /// Time taken to handle (milliseconds).
    pub elapsed_ms: u64,
}

/// Robustness report — how well a piece of code withstands adversarial inputs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobustnessReport {
    /// Stable identifier.
    pub id: String,
    /// Function or unit being tested.
    pub target: String,
    /// All adversarial inputs that were generated.
    pub inputs: Vec<AdversarialInput>,
    /// Results for each input.
    pub results: Vec<AdversarialResult>,
    /// Overall robustness score in `[0.0, 1.0]` (1.0 = perfectly robust).
    pub score: f32,
    /// Number of inputs that broke the code.
    pub failures: usize,
    /// Total inputs tested.
    pub total: usize,
    /// Recommendations to improve robustness.
    pub recommendations: Vec<String>,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl RobustnessReport {
    /// Returns the failure rate in `[0.0, 1.0]`.
    #[must_use]
    pub fn failure_rate(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.failures as f32 / self.total as f32
        }
    }

    /// Returns inputs that broke the code.
    pub fn breaking_inputs(&self) -> impl Iterator<Item = &AdversarialResult> {
        self.results.iter().filter(|r| !r.handled)
    }
}

/// An edge case identified by the explorer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeCase {
    /// Stable identifier.
    pub id: String,
    /// Description of the edge case.
    pub description: String,
    /// Example input that triggers it.
    pub example: String,
    /// Whether the code currently handles this edge case.
    pub handled: bool,
    /// Recommendation.
    pub recommendation: String,
    /// Confidence in `[0.0, 1.0]`.
    pub confidence: f32,
    /// Severity if not handled.
    pub severity: Severity,
    /// Tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Result of edge-case exploration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeCaseReport {
    /// Stable identifier.
    pub id: String,
    /// Function or unit analyzed.
    pub target: String,
    /// Edge cases identified.
    pub cases: Vec<EdgeCase>,
    /// Coverage score in `[0.0, 1.0]`.
    pub coverage: f32,
    /// Unhandled cases.
    pub unhandled: usize,
    /// Total cases.
    pub total: usize,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl EdgeCaseReport {
    /// Returns unhandled edge cases.
    pub fn unhandled_cases(&self) -> impl Iterator<Item = &EdgeCase> {
        self.cases.iter().filter(|c| !c.handled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adversarial_input_break_likelihood_clamped() {
        let i = AdversarialInput::new(AdversarialCategory::Empty, "empty string", "", "x", 1.5);
        assert_eq!(i.break_likelihood, 1.0);
    }

    #[test]
    fn robustness_failure_rate() {
        let r = RobustnessReport {
            id: "x".into(),
            target: "f".into(),
            inputs: vec![],
            results: vec![],
            score: 0.8,
            failures: 2,
            total: 10,
            recommendations: vec![],
            timestamp: chrono::Utc::now(),
        };
        assert!((r.failure_rate() - 0.2).abs() < 0.001);
    }

    #[test]
    fn all_categories_count() {
        assert_eq!(AdversarialCategory::all().len(), 11);
    }
}
