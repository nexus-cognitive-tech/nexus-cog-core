//! Thought-chain types.

use serde::{Deserialize, Serialize};

use super::common::Confidence;

/// Type of thought within a thought chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThoughtType {
    /// Identify the actual problem.
    Problem,
    /// Analyze dependencies and risks.
    Analysis,
    /// Form a hypothesis about the solution.
    Hypothesis,
    /// Make a decision.
    Decision,
    /// Implement the solution.
    Implementation,
    /// Verify the implementation.
    Verification,
    /// Reflect on what was learned.
    Reflection,
    /// Question / doubt about current reasoning.
    Question,
    /// Free-form observation.
    Observation,
    /// Note an assumption being made.
    Assumption,
}

impl ThoughtType {
    /// Stable identifier.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Problem => "problem",
            Self::Analysis => "analysis",
            Self::Hypothesis => "hypothesis",
            Self::Decision => "decision",
            Self::Implementation => "implementation",
            Self::Verification => "verification",
            Self::Reflection => "reflection",
            Self::Question => "question",
            Self::Observation => "observation",
            Self::Assumption => "assumption",
        }
    }
}

/// A single node in a thought chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThoughtNode {
    /// Stable identifier.
    pub id: String,
    /// Thought type.
    pub thought_type: ThoughtType,
    /// Content of the thought.
    pub content: String,
    /// Confidence in `[0.0, 1.0]`.
    pub confidence: Confidence,
    /// Children node IDs.
    pub children: Vec<String>,
    /// Parent node ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Timestamp when the thought was recorded.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Optional tags for grouping / search.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl ThoughtNode {
    /// Construct a new thought node.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        thought_type: ThoughtType,
        content: impl Into<String>,
        confidence: Confidence,
    ) -> Self {
        Self {
            id: id.into(),
            thought_type,
            content: content.into(),
            confidence,
            children: Vec::new(),
            parent: None,
            timestamp: chrono::Utc::now(),
            tags: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thought_type_ids_are_stable() {
        assert_eq!(ThoughtType::Problem.id(), "problem");
        assert_eq!(ThoughtType::Reflection.id(), "reflection");
    }
}
