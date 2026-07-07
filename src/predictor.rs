//! Success-predictor types.

use serde::{Deserialize, Serialize};

use super::learner::TaskComplexity;

/// A historical task used for prediction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalTask {
    /// Hash of the task features.
    pub task_hash: u64,
    /// Approach used.
    pub approach: String,
    /// Tools used.
    pub tools: Vec<String>,
    /// Number of rounds taken.
    pub rounds: usize,
    /// Whether the task succeeded.
    pub success: bool,
    /// Quality score.
    pub quality: f32,
    /// Complexity.
    pub complexity: TaskComplexity,
    /// When the task happened (Unix seconds).
    pub timestamp: i64,
}

/// A prediction for a new task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prediction {
    /// Confidence in the prediction itself in `[0.0, 1.0]`.
    pub confidence: f32,
    /// Probability of success in `[0.0, 1.0]`.
    pub success_probability: f32,
    /// Predicted approach name.
    pub predicted_approach: String,
    /// Predicted tools (ordered by expected utility).
    pub predicted_tools: Vec<String>,
    /// Predicted number of rounds.
    pub predicted_rounds: usize,
    /// Risk factors identified.
    pub risk_factors: Vec<String>,
    /// Predicted complexity.
    pub complexity: TaskComplexity,
    /// Whether the predictor has enough data to make a confident prediction.
    pub has_sufficient_data: bool,
    /// Free-form notes about the prediction.
    #[serde(default)]
    pub notes: Vec<String>,
}

impl Prediction {
    /// A cold-start prediction used when no history is available.
    #[must_use]
    pub fn cold_start(available_tools: &[String]) -> Self {
        Self {
            confidence: 0.3,
            success_probability: 0.5,
            predicted_approach: "explore".into(),
            predicted_tools: available_tools.iter().take(3).cloned().collect(),
            predicted_rounds: 5,
            risk_factors: vec!["No similar past tasks".into()],
            complexity: TaskComplexity::Moderate,
            has_sufficient_data: false,
            notes: vec!["Cold start: no historical data".into()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cold_start_returns_safe_defaults() {
        let p = Prediction::cold_start(&["a".into(), "b".into(), "c".into(), "d".into()]);
        assert_eq!(p.predicted_tools.len(), 3);
        assert!(!p.has_sufficient_data);
        assert!(p.confidence < 0.5);
    }
}
