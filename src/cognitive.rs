//! Cognitive-scaffold protocol types.

use serde::{Deserialize, Serialize};

/// Phase of the cognitive scaffold protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaffoldPhase {
    /// Understand the actual problem.
    Understand,
    /// Analyze dependencies and risks.
    Analyze,
    /// Design the minimal solution.
    Design,
    /// Implement the solution.
    Implement,
    /// Verify the implementation.
    Verify,
    /// Reflect on the solution and what was learned.
    Reflect,
}

impl ScaffoldPhase {
    /// Returns all phases in order.
    #[must_use]
    pub const fn all() -> [ScaffoldPhase; 6] {
        [
            Self::Understand,
            Self::Analyze,
            Self::Design,
            Self::Implement,
            Self::Verify,
            Self::Reflect,
        ]
    }

    /// Returns the 1-indexed position of this phase in the protocol.
    #[must_use]
    pub const fn index(self) -> u8 {
        match self {
            Self::Understand => 1,
            Self::Analyze => 2,
            Self::Design => 3,
            Self::Implement => 4,
            Self::Verify => 5,
            Self::Reflect => 6,
        }
    }

    /// Short label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Understand => "UNDERSTAND",
            Self::Analyze => "ANALYZE",
            Self::Design => "DESIGN",
            Self::Implement => "IMPLEMENT",
            Self::Verify => "VERIFY",
            Self::Reflect => "REFLECT",
        }
    }
}

/// System instruction emitted by the scaffold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScaffoldPrompt {
    /// Phase this prompt targets.
    pub phase: ScaffoldPhase,
    /// Full system instruction.
    pub system_instruction: String,
    /// User prompt with task + context.
    pub user_prompt: String,
    /// Required thought types the model should produce.
    pub required_thoughts: Vec<super::thought::ThoughtType>,
    /// Verification criteria.
    pub verification_criteria: Vec<String>,
}

/// Quality indicators extracted from a model response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityIndicators {
    /// Response handles errors with `Result`/`Option`/`?`/`match`.
    pub has_error_handling: bool,
    /// Response addresses edge cases.
    pub has_edge_cases: bool,
    /// Response contains documentation (`///` or `//!`).
    pub has_documentation: bool,
    /// Response does not contain `TODO` / `FIXME` / `HACK`.
    pub has_minimal_solution: bool,
    /// Response contains explicit reasoning (`because`, `therefore`, `since`).
    pub has_reasoning: bool,
    /// Response references existing patterns.
    pub references_patterns: bool,
    /// Response mentions alternatives considered.
    pub considers_alternatives: bool,
}

impl QualityIndicators {
    /// Count of indicators that are `true`.
    #[must_use]
    pub fn count(&self) -> u32 {
        u32::from(self.has_error_handling)
            + u32::from(self.has_edge_cases)
            + u32::from(self.has_documentation)
            + u32::from(self.has_minimal_solution)
            + u32::from(self.has_reasoning)
            + u32::from(self.references_patterns)
            + u32::from(self.considers_alternatives)
    }

    /// Fraction of indicators that are `true` in `[0.0, 1.0]`.
    #[must_use]
    pub fn fraction(&self) -> f32 {
        self.count() as f32 / 7.0
    }
}

/// Analysis of a model's response against the scaffold protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScaffoldAnalysis {
    /// Phase coverage — fraction of scaffold phases that appeared in the response.
    pub phase_score: f32,
    /// Per-phase presence.
    pub phase_presence: indexmap::IndexMap<String, bool>,
    /// Quality indicators.
    pub quality_indicators: QualityIndicators,
    /// Suggestions to improve the response.
    pub suggestions: Vec<String>,
    /// Confidence in the analysis itself, in `[0.0, 1.0]`.
    pub confidence: f32,
}

impl ScaffoldAnalysis {
    /// Returns the missing phases.
    #[must_use]
    pub fn missing_phases(&self) -> Vec<ScaffoldPhase> {
        ScaffoldPhase::all()
            .into_iter()
            .filter(|p| !self.phase_presence.get(p.label()).copied().unwrap_or(false))
            .collect()
    }

    /// Returns `true` if every phase was present.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.missing_phases().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_indices_are_one_indexed() {
        assert_eq!(ScaffoldPhase::Understand.index(), 1);
        assert_eq!(ScaffoldPhase::Reflect.index(), 6);
    }

    #[test]
    fn phase_all_returns_six() {
        assert_eq!(ScaffoldPhase::all().len(), 6);
    }

    #[test]
    fn quality_count_max_is_seven() {
        let q = QualityIndicators {
            has_error_handling: true,
            has_edge_cases: true,
            has_documentation: true,
            has_minimal_solution: true,
            has_reasoning: true,
            references_patterns: true,
            considers_alternatives: true,
        };
        assert_eq!(q.count(), 7);
        assert!((q.fraction() - 1.0).abs() < 0.001);
    }
}
