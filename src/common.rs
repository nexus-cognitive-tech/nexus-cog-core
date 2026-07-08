//! Common primitives shared across the cognitive stack.

use serde::{Deserialize, Serialize};

/// Severity level used by risk reports, architecture issues, and intent drift.
///
/// Ordered from least to most severe so that [`Ord`] reflects severity.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[non_exhaustive]
pub enum Severity {
    /// Informational finding; no action required.
    #[default]
    Info,
    /// Low severity.
    Low,
    /// Medium severity.
    Medium,
    /// Warning; should be addressed but not blocking.
    Warning,
    /// High severity.
    High,
    /// Error; must be addressed before merge.
    Error,
    /// Critical; security or correctness impact, blocking.
    Critical,
}

impl Severity {
    /// Returns a short human label (e.g. `"INFO"`, `"CRIT"`).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Low => "LOW",
            Self::Medium => "MED",
            Self::Warning => "WARN",
            Self::High => "HIGH",
            Self::Error => "ERROR",
            Self::Critical => "CRIT",
        }
    }

    /// Returns a numeric score in `[0.0, 1.0]` where `1.0` is most severe.
    #[must_use]
    pub const fn score(self) -> f32 {
        match self {
            Self::Info => 0.1,
            Self::Low => 0.2,
            Self::Medium => 0.3,
            Self::Warning => 0.4,
            Self::High => 0.6,
            Self::Error => 0.8,
            Self::Critical => 1.0,
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Source-code position (line + column, both 1-indexed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// 1-indexed line number.
    pub line: u32,
    /// 1-indexed column number.
    pub column: u32,
}

impl Position {
    /// Create a new position.
    #[must_use]
    pub const fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Inclusive byte or character range within a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range {
    /// Start position (inclusive).
    pub start: Position,
    /// End position (inclusive).
    pub end: Position,
}

impl Range {
    /// Create a new range.
    #[must_use]
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Single-line range.
    #[must_use]
    pub const fn line(line: u32, start_col: u32, end_col: u32) -> Self {
        Self {
            start: Position::new(line, start_col),
            end: Position::new(line, end_col),
        }
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

/// Programming language detected (or specified) for a piece of code.
///
/// Extensible via [`Language::Other`] so new languages can be added without breaking changes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Language {
    /// Rust source code.
    Rust,
    /// TypeScript source code.
    TypeScript,
    /// JavaScript source code.
    JavaScript,
    /// Python source code.
    Python,
    /// Go source code.
    Go,
    /// C source code.
    C,
    /// C++ source code.
    Cpp,
    /// Java source code.
    Java,
    /// Kotlin source code.
    Kotlin,
    /// Swift source code.
    Swift,
    /// Ruby source code.
    Ruby,
    /// PHP source code.
    Php,
    /// Shell / Bash script.
    Shell,
    /// HTML markup.
    Html,
    /// CSS stylesheet.
    Css,
    /// SQL query.
    Sql,
    /// Markdown documentation.
    Markdown,
    /// TOML configuration.
    Toml,
    /// YAML configuration.
    Yaml,
    /// JSON document.
    Json,
    /// Plain text (no specific language).
    Plain,
    /// Other language identified by a free-form string.
    Other(String),
}

impl Language {
    /// Construct from a file extension (lowercased).
    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "rs" => Self::Rust,
            "ts" | "tsx" | "mts" | "cts" => Self::TypeScript,
            "js" | "jsx" | "mjs" | "cjs" => Self::JavaScript,
            "py" | "pyi" | "pyw" => Self::Python,
            "go" => Self::Go,
            "c" | "h" => Self::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => Self::Cpp,
            "java" => Self::Java,
            "kt" | "kts" => Self::Kotlin,
            "swift" => Self::Swift,
            "rb" => Self::Ruby,
            "php" => Self::Php,
            "sh" | "bash" | "zsh" | "fish" => Self::Shell,
            "html" | "htm" => Self::Html,
            "css" => Self::Css,
            "sql" => Self::Sql,
            "md" | "markdown" => Self::Markdown,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "json" => Self::Json,
            "txt" => Self::Plain,
            other => Self::Other(other.to_string()),
        }
    }

    /// Construct from a human-readable language name (case-insensitive).
    ///
    /// Falls back to [`Self::Other`] for unrecognized names.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "rust" | "rs" => Self::Rust,
            "typescript" | "ts" | "tsx" => Self::TypeScript,
            "javascript" | "js" | "jsx" => Self::JavaScript,
            "python" | "py" => Self::Python,
            "go" | "golang" => Self::Go,
            "c" => Self::C,
            "c++" | "cpp" | "cxx" => Self::Cpp,
            "java" => Self::Java,
            "kotlin" | "kt" => Self::Kotlin,
            "swift" => Self::Swift,
            "ruby" | "rb" => Self::Ruby,
            "php" => Self::Php,
            "shell" | "sh" | "bash" => Self::Shell,
            "html" => Self::Html,
            "css" => Self::Css,
            "sql" => Self::Sql,
            "markdown" | "md" => Self::Markdown,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "json" => Self::Json,
            other => Self::Other(other.to_string()),
        }
    }

    /// Returns a stable lowercase identifier suitable for indexing.
    #[must_use]
    pub fn id(&self) -> String {
        match self {
            Self::Rust => "rust".to_string(),
            Self::TypeScript => "typescript".to_string(),
            Self::JavaScript => "javascript".to_string(),
            Self::Python => "python".to_string(),
            Self::Go => "go".to_string(),
            Self::C => "c".to_string(),
            Self::Cpp => "cpp".to_string(),
            Self::Java => "java".to_string(),
            Self::Kotlin => "kotlin".to_string(),
            Self::Swift => "swift".to_string(),
            Self::Ruby => "ruby".to_string(),
            Self::Php => "php".to_string(),
            Self::Shell => "shell".to_string(),
            Self::Html => "html".to_string(),
            Self::Css => "css".to_string(),
            Self::Sql => "sql".to_string(),
            Self::Markdown => "markdown".to_string(),
            Self::Toml => "toml".to_string(),
            Self::Yaml => "yaml".to_string(),
            Self::Json => "json".to_string(),
            Self::Plain => "plain".to_string(),
            Self::Other(s) => s.to_lowercase(),
        }
    }

    /// Returns `true` for languages with strict type systems where `unwrap`/`expect`
    /// warnings are appropriate.
    #[must_use]
    pub fn is_strict_typed(&self) -> bool {
        matches!(
            self,
            Self::Rust | Self::TypeScript | Self::Go | Self::Java | Self::Kotlin | Self::Swift
        )
    }

    /// Returns `true` for languages where memory safety is a primary concern.
    #[must_use]
    pub fn is_memory_sensitive(&self) -> bool {
        matches!(self, Self::Rust | Self::C | Self::Cpp)
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.id())
    }
}

/// Identifier for a file within a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileId(pub String);

impl FileId {
    /// Construct a new file ID.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Borrow the underlying path.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for FileId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for FileId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Identifier for a specific entity (function, struct, …) within a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymbolId {
    /// File containing the symbol.
    pub file: FileId,
    /// Qualified name of the symbol.
    pub name: String,
    /// 1-indexed line where the symbol starts.
    pub line: u32,
}

impl SymbolId {
    /// Construct a new symbol ID.
    #[must_use]
    pub fn new(file: impl Into<FileId>, name: impl Into<String>, line: u32) -> Self {
        Self {
            file: file.into(),
            name: name.into(),
            line,
        }
    }
}

impl std::fmt::Display for SymbolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}@{}", self.file, self.name, self.line)
    }
}

/// Bounded confidence score in `[0.0, 1.0]` used throughout the cognitive stack.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Confidence(f32);

impl Confidence {
    /// Minimum confidence.
    pub const MIN: f32 = 0.0;
    /// Maximum confidence.
    pub const MAX: f32 = 1.0;

    /// Construct a new confidence, clamping into `[0.0, 1.0]`.
    #[must_use]
    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    /// Raw clamped value.
    #[must_use]
    pub const fn value(self) -> f32 {
        self.0
    }

    /// Returns `true` if confidence is at or above the threshold.
    #[must_use]
    pub fn is_at_least(self, threshold: f32) -> bool {
        self.0 >= threshold
    }

    /// Returns `true` if confidence is below the threshold (uncertain).
    #[must_use]
    pub fn is_uncertain(self, threshold: f32) -> bool {
        self.0 < threshold
    }
}

impl Default for Confidence {
    fn default() -> Self {
        Self(0.5)
    }
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}%", self.0 * 100.0)
    }
}

impl From<f32> for Confidence {
    fn from(v: f32) -> Self {
        Self::new(v)
    }
}

impl From<f64> for Confidence {
    fn from(v: f64) -> Self {
        Self::new(v as f32)
    }
}

/// Bounded quality score in `[0.0, 1.0]`. Semantically distinct from confidence:
/// confidence = how sure we are of the result; quality = how good the result is.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Quality(f32);

impl Quality {
    /// Minimum quality.
    pub const MIN: f32 = 0.0;
    /// Maximum quality.
    pub const MAX: f32 = 1.0;

    /// Construct a new quality score, clamping into `[0.0, 1.0]`.
    #[must_use]
    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    /// Raw clamped value.
    #[must_use]
    pub const fn value(self) -> f32 {
        self.0
    }
}

impl Default for Quality {
    fn default() -> Self {
        Self(0.5)
    }
}

impl std::fmt::Display for Quality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}%", self.0 * 100.0)
    }
}

impl From<f32> for Quality {
    fn from(v: f32) -> Self {
        Self::new(v)
    }
}

/// Result of a yes/no question with associated confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Yes, with high confidence.
    Yes,
    /// No, with high confidence.
    No,
    /// Mixed or insufficient evidence.
    Mixed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ord_reflects_severity() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Critical);
    }

    #[test]
    fn severity_label_is_stable() {
        assert_eq!(Severity::Info.label(), "INFO");
        assert_eq!(Severity::Warning.label(), "WARN");
        assert_eq!(Severity::Error.label(), "ERROR");
        assert_eq!(Severity::Critical.label(), "CRIT");
    }

    #[test]
    fn language_from_extension_known() {
        assert_eq!(Language::from_extension("rs"), Language::Rust);
        assert_eq!(Language::from_extension("TS"), Language::TypeScript);
        assert_eq!(Language::from_extension("py"), Language::Python);
        assert_eq!(Language::from_extension("go"), Language::Go);
        assert_eq!(Language::from_extension("cpp"), Language::Cpp);
        assert_eq!(Language::from_extension("kts"), Language::Kotlin);
    }

    #[test]
    fn language_from_extension_unknown_is_other() {
        assert_eq!(
            Language::from_extension("xyz"),
            Language::Other("xyz".to_string())
        );
    }

    #[test]
    fn language_id_is_lowercase() {
        assert_eq!(Language::Rust.id(), "rust");
        assert_eq!(Language::Other("TypeScript".to_string()).id(), "typescript");
    }

    #[test]
    fn confidence_is_clamped() {
        assert_eq!(Confidence::new(-0.5).value(), 0.0);
        assert_eq!(Confidence::new(1.5).value(), 1.0);
        assert_eq!(Confidence::new(0.42).value(), 0.42);
    }

    #[test]
    fn confidence_is_uncertain_below_threshold() {
        assert!(Confidence::new(0.3).is_uncertain(0.5));
        assert!(!Confidence::new(0.7).is_uncertain(0.5));
    }

    #[test]
    fn quality_is_clamped() {
        assert_eq!(Quality::new(-1.0).value(), 0.0);
        assert_eq!(Quality::new(2.0).value(), 1.0);
        assert_eq!(Quality::new(0.8).value(), 0.8);
    }

    #[test]
    fn position_displays_line_col() {
        assert_eq!(Position::new(12, 4).to_string(), "12:4");
    }

    #[test]
    fn range_displays_start_dash_end() {
        let r = Range::line(3, 5, 10);
        assert_eq!(r.to_string(), "3:5-3:10");
    }

    #[test]
    fn file_id_roundtrips() {
        let id = FileId::new("src/lib.rs");
        assert_eq!(id.as_str(), "src/lib.rs");
        assert_eq!(id.to_string(), "src/lib.rs");
    }

    #[test]
    fn symbol_id_displays_qualified() {
        let s = SymbolId::new("src/lib.rs", "foo::bar", 42);
        assert_eq!(s.to_string(), "src/lib.rs::foo::bar@42");
    }
}
