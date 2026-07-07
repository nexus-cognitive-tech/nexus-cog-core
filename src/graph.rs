//! Code-graph types.

use serde::{Deserialize, Serialize};

/// Type of node in a code graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// Function definition.
    Function,
    /// Struct definition.
    Struct,
    /// Enum definition.
    Enum,
    /// Trait definition.
    Trait,
    /// Impl block.
    Impl,
    /// Module declaration.
    Module,
    /// Constant or `static`.
    Constant,
    /// Type alias.
    TypeAlias,
    /// Macro definition.
    Macro,
    /// Free-standing file.
    File,
}

impl NodeType {
    /// Stable lowercase identifier.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Impl => "impl",
            Self::Module => "module",
            Self::Constant => "constant",
            Self::TypeAlias => "type_alias",
            Self::Macro => "macro",
            Self::File => "file",
        }
    }
}

/// A node in the code graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphNode {
    /// Stable identifier (e.g. `"file::path::function_name@line"`).
    pub id: String,
    /// Node type.
    pub node_type: NodeType,
    /// Symbol name.
    pub name: String,
    /// File containing the symbol.
    pub file: String,
    /// 1-indexed line.
    pub line: u32,
    /// Per-node complexity (e.g. cyclomatic for functions).
    pub complexity: f32,
    /// Outgoing dependencies (other node IDs).
    pub dependencies: Vec<String>,
    /// Visibility (`"pub"`, `"pub(crate)"`, `""`).
    #[serde(default)]
    pub visibility: String,
    /// Optional signature for functions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl GraphNode {
    /// Construct a new graph node.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        node_type: NodeType,
        name: impl Into<String>,
        file: impl Into<String>,
        line: u32,
        complexity: f32,
    ) -> Self {
        Self {
            id: id.into(),
            node_type,
            name: name.into(),
            file: file.into(),
            line,
            complexity,
            dependencies: Vec::new(),
            visibility: String::new(),
            signature: None,
        }
    }
}

/// Type of edge in a code graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    /// Function A calls function B.
    Calls,
    /// Code uses a type.
    Uses,
    /// Struct implements a trait.
    Implements,
    /// Struct extends another.
    Extends,
    /// Module A contains symbol B.
    Contains,
    /// A imports B.
    Imports,
}

impl EdgeType {
    /// Stable identifier.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Calls => "calls",
            Self::Uses => "uses",
            Self::Implements => "implements",
            Self::Extends => "extends",
            Self::Contains => "contains",
            Self::Imports => "imports",
        }
    }
}

/// A directed edge in the code graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Edge type.
    pub edge_type: EdgeType,
    /// Weight in `[0.0, 1.0]` — higher means stronger coupling.
    pub weight: f32,
    /// Optional evidence (e.g. the line that triggered this edge).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

/// Aggregated metrics for a code graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphMetrics {
    /// Total number of nodes.
    pub total_nodes: usize,
    /// Total number of edges.
    pub total_edges: usize,
    /// Average complexity across all nodes.
    pub avg_complexity: f32,
    /// Maximum dependency depth.
    pub max_depth: usize,
    /// Total cyclomatic complexity.
    pub cyclomatic_complexity: usize,
    /// Average coupling (edges per node, external).
    pub coupling: f32,
    /// Cohesion in `[0.0, 1.0]`.
    pub cohesion: f32,
    /// Files identified as god modules (>30% of nodes).
    pub god_modules: Vec<String>,
    /// Top hotspots (nodes with most incoming edges).
    pub hotspots: Vec<String>,
}

impl GraphMetrics {
    /// Returns `true` if there are no metrics available (empty graph).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_nodes == 0
    }
}

/// The full code graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeGraph {
    /// All nodes.
    pub nodes: Vec<GraphNode>,
    /// All edges.
    pub edges: Vec<GraphEdge>,
    /// Aggregated metrics.
    pub metrics: GraphMetrics,
    /// Stable identifier.
    pub id: String,
    /// Timestamp when the graph was built.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl CodeGraph {
    /// Returns nodes of a specific type.
    pub fn nodes_of_type(&self, node_type: NodeType) -> impl Iterator<Item = &GraphNode> {
        self.nodes.iter().filter(move |n| n.node_type == node_type)
    }

    /// Returns the node with the given ID.
    #[must_use]
    pub fn node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Returns outgoing edges from a node.
    pub fn outgoing(&self, id: &str) -> impl Iterator<Item = &GraphEdge> {
        self.edges.iter().filter(move |e| e.from == id)
    }

    /// Returns incoming edges to a node.
    pub fn incoming(&self, id: &str) -> impl Iterator<Item = &GraphEdge> {
        self.edges.iter().filter(move |e| e.to == id)
    }

    /// Returns the dependencies (direct + transitive) of a node.
    #[must_use]
    pub fn dependencies_of(&self, id: &str) -> std::collections::HashSet<String> {
        let mut deps = std::collections::HashSet::new();
        let mut stack = vec![id.to_string()];
        while let Some(current) = stack.pop() {
            for edge in self.outgoing(&current) {
                if deps.insert(edge.to.clone()) {
                    stack.push(edge.to.clone());
                }
            }
        }
        deps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build() -> CodeGraph {
        CodeGraph {
            nodes: vec![
                GraphNode::new("a", NodeType::Function, "a", "f.rs", 1, 1.0),
                GraphNode::new("b", NodeType::Function, "b", "f.rs", 10, 1.0),
                GraphNode::new("c", NodeType::Function, "c", "g.rs", 1, 1.0),
            ],
            edges: vec![
                GraphEdge {
                    from: "a".into(),
                    to: "b".into(),
                    edge_type: EdgeType::Calls,
                    weight: 1.0,
                    evidence: None,
                },
                GraphEdge {
                    from: "b".into(),
                    to: "c".into(),
                    edge_type: EdgeType::Calls,
                    weight: 1.0,
                    evidence: None,
                },
            ],
            metrics: GraphMetrics {
                total_nodes: 3,
                total_edges: 2,
                avg_complexity: 1.0,
                max_depth: 2,
                cyclomatic_complexity: 4,
                coupling: 2.0 / 3.0,
                cohesion: 0.0,
                god_modules: vec![],
                hotspots: vec![],
            },
            id: "x".into(),
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn node_type_ids_are_stable() {
        assert_eq!(NodeType::Function.id(), "function");
        assert_eq!(NodeType::Struct.id(), "struct");
    }

    #[test]
    fn dependencies_of_returns_transitive() {
        let g = build();
        let deps = g.dependencies_of("a");
        assert!(deps.contains("b"));
        assert!(deps.contains("c"));
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn outgoing_filters_correctly() {
        let g = build();
        let outgoing: Vec<_> = g.outgoing("a").collect();
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].to, "b");
    }

    #[test]
    fn incoming_filters_correctly() {
        let g = build();
        let incoming: Vec<_> = g.incoming("c").collect();
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].from, "b");
    }
}
