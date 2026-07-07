//! Pattern library types.

use serde::{Deserialize, Serialize};

/// Category of pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternCategory {
    /// Error handling patterns (`Result`, `?`, `Option`).
    ErrorHandling,
    /// Async / concurrency patterns.
    Async,
    /// Builder pattern.
    Builder,
    /// Factory pattern.
    Factory,
    /// Observer pattern.
    Observer,
    /// State machine.
    StateMachine,
    /// Strategy pattern.
    Strategy,
    /// Repository pattern.
    Repository,
    /// Decorator pattern.
    Decorator,
    /// Iterator / functional patterns.
    Iterator,
    /// Resource acquisition is initialization.
    Raii,
    /// Custom pattern with a free-form name.
    Custom(String),
}

impl PatternCategory {
    /// Stable identifier.
    #[must_use]
    pub fn id(&self) -> String {
        match self {
            Self::ErrorHandling => "error_handling".to_string(),
            Self::Async => "async".to_string(),
            Self::Builder => "builder".to_string(),
            Self::Factory => "factory".to_string(),
            Self::Observer => "observer".to_string(),
            Self::StateMachine => "state_machine".to_string(),
            Self::Strategy => "strategy".to_string(),
            Self::Repository => "repository".to_string(),
            Self::Decorator => "decorator".to_string(),
            Self::Iterator => "iterator".to_string(),
            Self::Raii => "raii".to_string(),
            Self::Custom(s) => s.to_lowercase().replace(' ', "_"),
        }
    }
}

/// Complexity of applying a pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternComplexity {
    /// Easy to apply; usually straightforward syntactic change.
    Low,
    /// Moderate; requires some refactoring.
    Medium,
    /// Hard; significant architectural change.
    High,
}

/// Context in which a pattern applies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternContext {
    /// Programming language.
    pub language: String,
    /// Optional framework (e.g. `"tokio"`, `"axum"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    /// Pattern complexity.
    pub complexity: PatternComplexity,
    /// Free-form tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl PatternContext {
    /// Construct a context for a specific language.
    #[must_use]
    pub fn for_language(language: impl Into<String>) -> Self {
        Self {
            language: language.into(),
            framework: None,
            complexity: PatternComplexity::Low,
            tags: Vec::new(),
        }
    }
}

/// A reusable code pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodePattern {
    /// Stable identifier (e.g. `"rust-result-propagation"`).
    pub id: String,
    /// Pattern category.
    pub pattern_type: PatternCategory,
    /// Short signature (e.g. `"fn -> Result<T, E>"`).
    pub signature: String,
    /// Worked examples of the pattern.
    pub examples: Vec<String>,
    /// Success rate in `[0.0, 1.0]`.
    pub success_rate: f32,
    /// Context in which this pattern applies.
    pub context: PatternContext,
    /// Description.
    #[serde(default)]
    pub description: String,
    /// When this pattern should NOT be used.
    #[serde(default)]
    pub anti_patterns: Vec<String>,
    /// Number of times this pattern has been matched.
    #[serde(default)]
    pub match_count: u64,
}

impl CodePattern {
    /// Construct a new pattern.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        pattern_type: PatternCategory,
        signature: impl Into<String>,
        success_rate: f32,
        context: PatternContext,
    ) -> Self {
        Self {
            id: id.into(),
            pattern_type,
            signature: signature.into(),
            examples: Vec::new(),
            success_rate: success_rate.clamp(0.0, 1.0),
            context,
            description: String::new(),
            anti_patterns: Vec::new(),
            match_count: 0,
        }
    }

    /// Returns `true` if this pattern is considered "proven" (`success_rate >= 0.85`
    /// and at least 5 matches).
    #[must_use]
    pub fn is_proven(&self) -> bool {
        self.success_rate >= 0.85 && self.match_count >= 5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_category_id_normalizes() {
        assert_eq!(PatternCategory::ErrorHandling.id(), "error_handling");
        assert_eq!(
            PatternCategory::Custom("My Pattern".into()).id(),
            "my_pattern"
        );
    }

    #[test]
    fn proven_requires_rate_and_volume() {
        let mut p = CodePattern::new(
            "x",
            PatternCategory::ErrorHandling,
            "x",
            0.95,
            PatternContext::for_language("rust"),
        );
        assert!(!p.is_proven());
        p.match_count = 10;
        assert!(p.is_proven());
        p.success_rate = 0.5;
        assert!(!p.is_proven());
    }

    #[test]
    fn success_rate_is_clamped() {
        let p = CodePattern::new(
            "x",
            PatternCategory::ErrorHandling,
            "x",
            5.0,
            PatternContext::for_language("rust"),
        );
        assert_eq!(p.success_rate, 1.0);
    }
}
