//! Adaptive-learner types.

use serde::{Deserialize, Serialize};

/// Complexity of a task — used by the predictor and learner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskComplexity {
    /// Single-line, trivial change.
    Trivial,
    /// A few lines, no new concepts.
    Simple,
    /// Multiple files, new types or functions.
    Moderate,
    /// Cross-cutting changes, new abstractions.
    Complex,
    /// Requires domain expertise or unfamiliar APIs.
    Expert,
}

impl TaskComplexity {
    /// Returns the estimated number of agent rounds for this complexity.
    #[must_use]
    pub const fn estimated_rounds(self) -> usize {
        match self {
            Self::Trivial => 1,
            Self::Simple => 3,
            Self::Moderate => 8,
            Self::Complex => 15,
            Self::Expert => 25,
        }
    }
}

/// Context surrounding a recorded interaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteractionContext {
    /// Programming language (e.g. `"rust"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Framework (e.g. `"tokio"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    /// Complexity bucket.
    pub complexity: TaskComplexity,
    /// IDs of similar past interactions.
    #[serde(default)]
    pub similar_past_tasks: Vec<String>,
}

impl Default for InteractionContext {
    fn default() -> Self {
        Self {
            language: None,
            framework: None,
            complexity: TaskComplexity::Moderate,
            similar_past_tasks: Vec::new(),
        }
    }
}

/// A recorded agent interaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interaction {
    /// Stable identifier.
    pub id: String,
    /// The original task description.
    pub task: String,
    /// Approach name (e.g. `"agent-loop"`, `"sub-agent-verify-then-fix"`).
    pub approach: String,
    /// Tools used (in order).
    pub tools_used: Vec<String>,
    /// Number of rounds taken.
    pub rounds: usize,
    /// Whether the task succeeded.
    pub success: bool,
    /// Quality score of the result in `[0.0, 1.0]`.
    pub quality_score: f32,
    /// Unix timestamp (seconds).
    pub timestamp: i64,
    /// Context.
    pub context: InteractionContext,
    /// Optional final output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Optional error message if failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Type of learning pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningPatternType {
    /// Tool X tends to work better than Y for tasks of type T.
    ToolPreference,
    /// Approach A tends to work better than B for complexity C.
    ApproachStyle,
    /// Recovery action for a particular error pattern.
    ErrorRecovery,
    /// Optimization discovered.
    Optimization,
    /// Coding style preference.
    CodeStyle,
    /// Architectural preference.
    Architecture,
}

/// A learned pattern extracted from past interactions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearningPattern {
    /// Pattern type.
    pub pattern_type: LearningPatternType,
    /// Trigger (e.g. task type, error message).
    pub trigger: String,
    /// Recommended action (e.g. tool name, approach).
    pub action: String,
    /// Confidence in `[0.0, 1.0]`.
    pub confidence: f32,
    /// Number of successful applications.
    pub success_count: u32,
    /// Number of failed applications.
    pub fail_count: u32,
    /// Timestamp of last update.
    pub last_updated: i64,
    /// Stable identifier.
    pub id: String,
    /// Tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A learned rule (more concrete than a pattern).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearnedRule {
    /// When to apply (free-form condition).
    pub condition: String,
    /// What to do.
    pub action: String,
    /// Confidence in `[0.0, 1.0]`.
    pub confidence: f32,
    /// Source description (e.g. `"learned from interactions 12, 14, 17"`).
    pub source: String,
    /// Stable identifier.
    pub id: String,
}

/// Aggregated learner statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearnerStats {
    /// Total recorded interactions.
    pub total_interactions: usize,
    /// Success rate in `[0.0, 1.0]`.
    pub success_rate: f32,
    /// Average quality score.
    pub avg_quality: f32,
    /// Number of learned patterns.
    pub patterns_learned: usize,
    /// Number of learned rules.
    pub rules_learned: usize,
    /// Per-tool success rate map.
    pub tool_success_rates: indexmap::IndexMap<String, f32>,
    /// Tasks grouped by complexity.
    pub by_complexity: indexmap::IndexMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complexity_round_estimates_increase() {
        assert!(
            TaskComplexity::Trivial.estimated_rounds() < TaskComplexity::Simple.estimated_rounds()
        );
        assert!(
            TaskComplexity::Simple.estimated_rounds() < TaskComplexity::Moderate.estimated_rounds()
        );
        assert!(
            TaskComplexity::Moderate.estimated_rounds()
                < TaskComplexity::Complex.estimated_rounds()
        );
        assert!(
            TaskComplexity::Complex.estimated_rounds() < TaskComplexity::Expert.estimated_rounds()
        );
    }
}
