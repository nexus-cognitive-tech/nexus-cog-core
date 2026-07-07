//! Risk report types.

use serde::{Deserialize, Serialize};

use super::common::{Position, Range, Severity};

/// Category of risk detected by the risk analyzer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskType {
    /// Security risk (e.g. SQL injection, hardcoded secrets, unsafe).
    Security,
    /// Performance risk (e.g. O(n²) on hot path, unnecessary clones).
    Performance,
    /// Reliability risk (e.g. unwrap, infinite loop, race condition).
    Reliability,
    /// Maintainability risk (e.g. god object, dead code, tight coupling).
    Maintainability,
    /// Scalability risk (e.g. unbounded growth, missing backpressure).
    Scalability,
}

impl RiskType {
    /// Returns all risk types.
    #[must_use]
    pub const fn all() -> [RiskType; 5] {
        [
            Self::Security,
            Self::Performance,
            Self::Reliability,
            Self::Maintainability,
            Self::Scalability,
        ]
    }

    /// Stable lowercase identifier.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Security => "security",
            Self::Performance => "performance",
            Self::Reliability => "reliability",
            Self::Maintainability => "maintainability",
            Self::Scalability => "scalability",
        }
    }
}

impl std::fmt::Display for RiskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id())
    }
}

/// A single detected risk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Risk {
    /// Type of risk.
    pub risk_type: RiskType,
    /// Severity.
    pub severity: Severity,
    /// Short identifier (e.g. `"hardcoded_secret"`).
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Free-form description of what was detected.
    pub description: String,
    /// Recommended mitigation.
    pub mitigation: String,
    /// Location of the risk in the source code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Range>,
    /// Original token that triggered the risk (e.g. the variable name that looks like a secret).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
}

impl Risk {
    /// Construct a new risk.
    #[must_use]
    pub fn new(
        risk_type: RiskType,
        severity: Severity,
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        mitigation: impl Into<String>,
    ) -> Self {
        Self {
            risk_type,
            severity,
            id: id.into(),
            name: name.into(),
            description: description.into(),
            mitigation: mitigation.into(),
            location: None,
            trigger: None,
        }
    }

    /// Attach a position to this risk.
    #[must_use]
    pub fn at(mut self, line: u32, column: u32) -> Self {
        self.location = Some(Range::line(line, column, column));
        self
    }

    /// Attach a trigger token.
    #[must_use]
    pub fn triggered_by(mut self, trigger: impl Into<String>) -> Self {
        self.trigger = Some(trigger.into());
        self
    }
}

/// Overall aggregated risk report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskReport {
    /// File or unit analyzed.
    pub file: String,
    /// All detected risks.
    pub risks: Vec<Risk>,
    /// Overall risk level — derived from the worst individual severity.
    pub overall: Severity,
    /// Recommended actions for the user / agent.
    pub recommendations: Vec<String>,
    /// Statistics about the risks found.
    pub stats: RiskStats,
    /// Stable identifier.
    pub id: String,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl RiskReport {
    /// Returns `true` if there is any risk at or above the given severity.
    #[must_use]
    pub fn has_risk_at_or_above(&self, threshold: Severity) -> bool {
        self.risks.iter().any(|r| r.severity >= threshold)
    }

    /// Returns risks of a specific type.
    pub fn risks_of_type(&self, risk_type: RiskType) -> impl Iterator<Item = &Risk> {
        self.risks.iter().filter(move |r| r.risk_type == risk_type)
    }

    /// Returns risks at or above the given severity.
    pub fn risks_at_or_above(&self, threshold: Severity) -> impl Iterator<Item = &Risk> {
        self.risks.iter().filter(move |r| r.severity >= threshold)
    }

    /// Returns the first risk at a specific line, if any.
    #[must_use]
    pub fn risk_at_line(&self, line: u32) -> Option<&Risk> {
        self.risks
            .iter()
            .find(|r| r.location.is_some_and(|loc| loc.start.line == line))
    }

    /// Returns the first risk at a specific position, if any.
    #[must_use]
    pub fn risk_at(&self, pos: Position) -> Option<&Risk> {
        self.risks
            .iter()
            .find(|r| r.location.is_some_and(|loc| loc.start.line == pos.line))
    }
}

/// Statistics aggregated from a risk report.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RiskStats {
    /// Total number of risks.
    pub total: usize,
    /// Number of risks per severity.
    pub by_severity: indexmap::IndexMap<String, usize>,
    /// Number of risks per type.
    pub by_type: indexmap::IndexMap<String, usize>,
    /// Total number of `Critical` risks.
    pub critical_count: usize,
    /// Total number of `Error` risks.
    pub error_count: usize,
}

impl RiskStats {
    /// Build statistics from a slice of risks.
    #[must_use]
    pub fn from_risks(risks: &[Risk]) -> Self {
        let mut stats = Self {
            total: risks.len(),
            ..Self::default()
        };
        for risk in risks {
            *stats
                .by_severity
                .entry(risk.severity.label().to_string())
                .or_insert(0) += 1;
            *stats
                .by_type
                .entry(risk.risk_type.id().to_string())
                .or_insert(0) += 1;
            match risk.severity {
                Severity::Critical => stats.critical_count += 1,
                Severity::Error => stats.error_count += 1,
                _ => {}
            }
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_risk(severity: Severity, risk_type: RiskType) -> Risk {
        Risk::new(risk_type, severity, "x", "x", "x", "x")
    }

    #[test]
    fn has_risk_at_or_above_filters_correctly() {
        let report = RiskReport {
            file: "x".into(),
            risks: vec![
                sample_risk(Severity::Warning, RiskType::Security),
                sample_risk(Severity::Critical, RiskType::Security),
            ],
            overall: Severity::Critical,
            recommendations: vec![],
            stats: RiskStats::default(),
            id: "x".into(),
            timestamp: chrono::Utc::now(),
        };
        assert!(report.has_risk_at_or_above(Severity::Warning));
        assert!(report.has_risk_at_or_above(Severity::Critical));
        // Info is the lowest severity, so any risk is at-or-above Info.
        assert!(report.has_risk_at_or_above(Severity::Info));
    }

    #[test]
    fn risk_at_line_returns_matching() {
        let mut r = sample_risk(Severity::Warning, RiskType::Security);
        r = r.at(42, 0);
        let report = RiskReport {
            file: "x".into(),
            risks: vec![r],
            overall: Severity::Warning,
            recommendations: vec![],
            stats: RiskStats::default(),
            id: "x".into(),
            timestamp: chrono::Utc::now(),
        };
        assert!(report.risk_at_line(42).is_some());
        assert!(report.risk_at_line(99).is_none());
    }

    #[test]
    fn stats_counts_severities_and_types() {
        let risks = vec![
            sample_risk(Severity::Critical, RiskType::Security),
            sample_risk(Severity::Critical, RiskType::Security),
            sample_risk(Severity::Warning, RiskType::Performance),
        ];
        let stats = RiskStats::from_risks(&risks);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.critical_count, 2);
        assert_eq!(stats.by_severity.get("CRIT"), Some(&2));
        assert_eq!(stats.by_type.get("security"), Some(&2));
        assert_eq!(stats.by_type.get("performance"), Some(&1));
    }
}
