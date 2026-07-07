//! Hypothesis-engine types.

use serde::{Deserialize, Serialize};

/// Status of a hypothesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisStatus {
    /// Hypothesis was proposed but not yet tested.
    Proposed,
    /// Hypothesis is currently being tested.
    Testing,
    /// Testing finished and one approach was picked.
    Completed,
    /// Hypothesis was rejected without testing.
    Rejected,
}

/// Estimated metrics for an approach (pre-implementation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimatedMetrics {
    /// Estimated complexity in `[0.0, 1.0]`.
    pub complexity: f32,
    /// Estimated performance in `[0.0, 1.0]`.
    pub performance: f32,
    /// Estimated readability in `[0.0, 1.0]`.
    pub readability: f32,
    /// Estimated maintainability in `[0.0, 1.0]`.
    pub maintainability: f32,
}

impl EstimatedMetrics {
    /// Returns the average across all dimensions.
    #[must_use]
    pub fn average(&self) -> f32 {
        (self.complexity + self.performance + self.readability + self.maintainability) / 4.0
    }
}

/// Actual measured metrics for an approach (post-implementation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActualMetrics {
    /// Execution time in milliseconds.
    pub execution_time_ms: f64,
    /// Memory consumption in bytes.
    pub memory_bytes: u64,
    /// Pass rate of the test suite.
    pub test_pass_rate: f32,
    /// Code coverage achieved by the tests.
    pub code_coverage: f32,
}

/// One of two competing approaches in a hypothesis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Approach {
    /// Name of the approach (e.g. `"Iterative with HashMap"`, `"Recursive with memoization"`).
    pub name: String,
    /// Source code implementing the approach.
    pub code: String,
    /// Rationale explaining why this approach was chosen.
    pub rationale: String,
    /// Pre-implementation metrics.
    pub estimated_metrics: EstimatedMetrics,
    /// Post-implementation metrics (filled after testing).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_metrics: Option<ActualMetrics>,
    /// Pros of this approach.
    #[serde(default)]
    pub pros: Vec<String>,
    /// Cons of this approach.
    #[serde(default)]
    pub cons: Vec<String>,
}

impl Approach {
    /// Construct a new approach.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        code: impl Into<String>,
        rationale: impl Into<String>,
        estimated: EstimatedMetrics,
    ) -> Self {
        Self {
            name: name.into(),
            code: code.into(),
            rationale: rationale.into(),
            estimated_metrics: estimated,
            actual_metrics: None,
            pros: Vec::new(),
            cons: Vec::new(),
        }
    }
}

/// Results of testing both approaches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestResults {
    /// Name of the winning approach.
    pub winner: String,
    /// Confidence in the result in `[0.0, 1.0]`.
    pub confidence: f32,
    /// Actual metrics for approach A.
    pub metrics_a: ActualMetrics,
    /// Actual metrics for approach B.
    pub metrics_b: ActualMetrics,
    /// Recommendation summary.
    pub recommendation: String,
    /// Deltas in favor of the winner.
    #[serde(default)]
    pub deltas: Vec<String>,
}

/// A hypothesis comparing two approaches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hypothesis {
    /// Stable identifier.
    pub id: String,
    /// Short title.
    pub title: String,
    /// Longer description.
    pub description: String,
    /// First approach.
    pub approach_a: Approach,
    /// Second approach.
    pub approach_b: Approach,
    /// Current status.
    pub status: HypothesisStatus,
    /// Test results (if completed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results: Option<TestResults>,
    /// Tags for grouping.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Timestamp when proposed.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Hypothesis {
    /// Returns `true` if the hypothesis has been resolved.
    #[must_use]
    pub fn is_resolved(&self) -> bool {
        matches!(
            self.status,
            HypothesisStatus::Completed | HypothesisStatus::Rejected
        )
    }

    /// Returns the winning approach, if any.
    #[must_use]
    pub fn winner(&self) -> Option<&Approach> {
        match (&self.status, &self.results) {
            (HypothesisStatus::Completed, Some(results)) => {
                if results.winner == self.approach_a.name {
                    Some(&self.approach_a)
                } else if results.winner == self.approach_b.name {
                    Some(&self.approach_b)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> Hypothesis {
        Hypothesis {
            id: "h1".into(),
            title: "two ways".into(),
            description: "x".into(),
            approach_a: Approach::new(
                "A",
                "fn a(){}",
                "rationale a",
                EstimatedMetrics {
                    complexity: 0.5,
                    performance: 0.7,
                    readability: 0.8,
                    maintainability: 0.6,
                },
            ),
            approach_b: Approach::new(
                "B",
                "fn b(){}",
                "rationale b",
                EstimatedMetrics {
                    complexity: 0.6,
                    performance: 0.6,
                    readability: 0.7,
                    maintainability: 0.7,
                },
            ),
            status: HypothesisStatus::Completed,
            results: Some(TestResults {
                winner: "A".into(),
                confidence: 0.9,
                metrics_a: ActualMetrics {
                    execution_time_ms: 10.0,
                    memory_bytes: 1024,
                    test_pass_rate: 1.0,
                    code_coverage: 0.95,
                },
                metrics_b: ActualMetrics {
                    execution_time_ms: 20.0,
                    memory_bytes: 2048,
                    test_pass_rate: 0.8,
                    code_coverage: 0.7,
                },
                recommendation: "Pick A".into(),
                deltas: vec!["A is 2x faster".into()],
            }),
            tags: vec![],
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn estimated_average() {
        let m = EstimatedMetrics {
            complexity: 0.8,
            performance: 0.6,
            readability: 0.4,
            maintainability: 0.2,
        };
        assert!((m.average() - 0.5).abs() < 0.001);
    }

    #[test]
    fn winner_returns_correct_approach() {
        let h = make();
        assert_eq!(h.winner().unwrap().name, "A");
    }

    #[test]
    fn unresolved_hypothesis_has_no_winner() {
        let mut h = make();
        h.status = HypothesisStatus::Proposed;
        h.results = None;
        assert!(h.winner().is_none());
        assert!(!h.is_resolved());
    }
}
