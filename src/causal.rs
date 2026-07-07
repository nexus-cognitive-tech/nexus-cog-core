//! Causal-graph types.
//!
//! These types support forward causal analysis ("if I change X, what breaks?"),
//! backward causal analysis ("why does this bug exist?"), counterfactual reasoning
//! ("what change would have prevented it?"), and pre-mortem analysis.

use serde::{Deserialize, Serialize};

use super::common::{Confidence, Severity};

/// Type of node in a causal graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalNodeType {
    /// A piece of code (function, struct, …).
    CodeEntity,
    /// A behavior / observable effect.
    Behavior,
    /// A user-visible feature.
    Feature,
    /// An invariant maintained by code.
    Invariant,
    /// An assumption (explicit or implicit).
    Assumption,
    /// A decision / design choice.
    Decision,
    /// A constraint (e.g. hardware, language limitation).
    Constraint,
    /// A bug.
    Bug,
    /// An external dependency.
    ExternalDep,
}

/// A node in the causal graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalNode {
    /// Stable identifier.
    pub id: String,
    /// Node type.
    pub node_type: CausalNodeType,
    /// Short name.
    pub name: String,
    /// Free-form description.
    pub description: String,
    /// File containing this node (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Line (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Confidence that this node exists / is correctly identified.
    pub confidence: Confidence,
    /// Tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Type of causal edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalEdgeType {
    /// A directly causes B.
    Causes,
    /// A enables / is a precondition for B.
    Enables,
    /// A prevents B.
    Prevents,
    /// A mitigates the impact of B.
    Mitigates,
    /// A is correlated with B (not causal, but useful for inference).
    Correlates,
}

impl CausalEdgeType {
    /// Returns `true` if this is a "positive" causal relationship.
    #[must_use]
    pub const fn is_positive(self) -> bool {
        matches!(self, Self::Causes | Self::Enables | Self::Mitigates)
    }
}

/// An edge in the causal graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalEdge {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Edge type.
    pub edge_type: CausalEdgeType,
    /// Causal strength in `[0.0, 1.0]`.
    pub strength: f32,
    /// Confidence in the relationship.
    pub confidence: Confidence,
    /// Evidence supporting this edge.
    #[serde(default)]
    pub evidence: Vec<String>,
}

/// The full causal graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalGraph {
    /// All nodes.
    pub nodes: Vec<CausalNode>,
    /// All edges.
    pub edges: Vec<CausalEdge>,
    /// Stable identifier.
    pub id: String,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Scope (e.g. project name).
    #[serde(default)]
    pub scope: String,
}

impl CausalGraph {
    /// Find a node by ID.
    #[must_use]
    pub fn node(&self, id: &str) -> Option<&CausalNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Forward causal closure — everything that could be caused by the given node.
    #[must_use]
    pub fn forward_closure(&self, id: &str) -> std::collections::HashSet<String> {
        let mut closure = std::collections::HashSet::new();
        let mut stack = vec![id.to_string()];
        while let Some(current) = stack.pop() {
            for edge in self.edges.iter().filter(|e| e.from == current) {
                if edge.edge_type.is_positive() && closure.insert(edge.to.clone()) {
                    stack.push(edge.to.clone());
                }
            }
        }
        closure
    }

    /// Backward causal closure — all possible causes of the given node.
    #[must_use]
    pub fn backward_closure(&self, id: &str) -> std::collections::HashSet<String> {
        let mut closure = std::collections::HashSet::new();
        let mut stack = vec![id.to_string()];
        while let Some(current) = stack.pop() {
            for edge in self.edges.iter().filter(|e| e.to == current) {
                if edge.edge_type.is_positive() && closure.insert(edge.from.clone()) {
                    stack.push(edge.from.clone());
                }
            }
        }
        closure
    }
}

/// A counterfactual hypothesis: "what change would have prevented this outcome?"
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Counterfactual {
    /// Stable identifier.
    pub id: String,
    /// The actual outcome that occurred (a node ID).
    pub outcome: String,
    /// The proposed counterfactual change.
    pub proposed_change: String,
    /// Why this change would have prevented the outcome.
    pub reasoning: String,
    /// Plausibility in `[0.0, 1.0]`.
    pub plausibility: f32,
    /// Supporting evidence.
    #[serde(default)]
    pub evidence: Vec<String>,
    /// Alternative counterfactuals considered.
    #[serde(default)]
    pub alternatives: Vec<String>,
}

/// A failure scenario in a pre-mortem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailureScenario {
    /// Stable identifier.
    pub id: String,
    /// Short title (e.g. "Database connection pool exhaustion").
    pub title: String,
    /// Detailed description.
    pub description: String,
    /// Likelihood in `[0.0, 1.0]`.
    pub likelihood: f32,
    /// Impact severity.
    pub impact: Severity,
    /// Time horizon in days (e.g. 30, 90, 365).
    pub time_horizon_days: u32,
    /// Early warning signs.
    #[serde(default)]
    pub warning_signs: Vec<String>,
    /// Concrete mitigations.
    #[serde(default)]
    pub mitigations: Vec<String>,
    /// Causal chain (node IDs leading to this failure).
    #[serde(default)]
    pub causal_chain: Vec<String>,
}

/// Result of a pre-mortem analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreMortemReport {
    /// Stable identifier.
    pub id: String,
    /// What is being pre-mortemed (e.g. "merging PR #1234", "deploying v2.0").
    pub subject: String,
    /// All failure scenarios identified.
    pub scenarios: Vec<FailureScenario>,
    /// Top N most critical scenarios (by `likelihood * impact`).
    pub top_risks: Vec<FailureScenario>,
    /// Overall risk score in `[0.0, 1.0]`.
    pub overall_risk: f32,
    /// Recommendations before proceeding.
    pub recommendations: Vec<String>,
}

/// Result of blast-radius analysis for a proposed change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlastRadius {
    /// Stable identifier.
    pub id: String,
    /// The changed entity.
    pub changed: String,
    /// All entities that could be affected (transitively).
    pub affected: Vec<String>,
    /// Number of affected entities.
    pub affected_count: usize,
    /// Risk score in `[0.0, 1.0]`.
    pub risk_score: f32,
    /// Breakdown of affected entities by type.
    pub by_type: indexmap::IndexMap<String, usize>,
    /// Recommendation.
    pub recommendation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_type_positive_classification() {
        assert!(CausalEdgeType::Causes.is_positive());
        assert!(CausalEdgeType::Enables.is_positive());
        assert!(!CausalEdgeType::Prevents.is_positive());
        assert!(!CausalEdgeType::Correlates.is_positive());
    }

    #[test]
    fn forward_closure_returns_descendants() {
        let g = CausalGraph {
            nodes: vec![
                CausalNode {
                    id: "a".into(),
                    node_type: CausalNodeType::CodeEntity,
                    name: "a".into(),
                    description: String::new(),
                    file: None,
                    line: None,
                    confidence: Confidence::new(1.0),
                    tags: vec![],
                },
                CausalNode {
                    id: "b".into(),
                    node_type: CausalNodeType::Behavior,
                    name: "b".into(),
                    description: String::new(),
                    file: None,
                    line: None,
                    confidence: Confidence::new(1.0),
                    tags: vec![],
                },
                CausalNode {
                    id: "c".into(),
                    node_type: CausalNodeType::Feature,
                    name: "c".into(),
                    description: String::new(),
                    file: None,
                    line: None,
                    confidence: Confidence::new(1.0),
                    tags: vec![],
                },
            ],
            edges: vec![
                CausalEdge {
                    from: "a".into(),
                    to: "b".into(),
                    edge_type: CausalEdgeType::Causes,
                    strength: 0.9,
                    confidence: Confidence::new(0.9),
                    evidence: vec![],
                },
                CausalEdge {
                    from: "b".into(),
                    to: "c".into(),
                    edge_type: CausalEdgeType::Enables,
                    strength: 0.7,
                    confidence: Confidence::new(0.7),
                    evidence: vec![],
                },
            ],
            id: "g".into(),
            timestamp: chrono::Utc::now(),
            scope: String::new(),
        };
        let closure = g.forward_closure("a");
        assert!(closure.contains("b"));
        assert!(closure.contains("c"));
        assert_eq!(closure.len(), 2);
    }
}
