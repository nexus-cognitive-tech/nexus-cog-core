//! Search result types.

use serde::{Deserialize, Serialize};

/// How a [`SearchResult`] matched the query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    /// Exact textual match (case-insensitive substring).
    Exact,
    /// Semantic similarity based on word overlap and structure.
    Semantic,
    /// Structural pattern match (e.g. both query and line contain `fn`, `struct`, …).
    Structural,
    /// Behavioral pattern match (e.g. both query and line contain control-flow keywords).
    Behavioral,
}

impl MatchType {
    /// Returns the score weight used to break ties.
    #[must_use]
    pub const fn weight(self) -> f32 {
        match self {
            Self::Exact => 1.0,
            Self::Semantic => 0.8,
            Self::Structural => 0.7,
            Self::Behavioral => 0.6,
        }
    }
}

/// A single search result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    /// File where the match was found.
    pub file: String,
    /// 1-indexed line number.
    pub line: u32,
    /// The matched line.
    pub content: String,
    /// Score in `[0.0, 1.0]` — higher is more relevant.
    pub score: f32,
    /// Surrounding context (a few lines before and after).
    pub context: String,
    /// How the match was found.
    pub match_type: MatchType,
    /// Optional offset within `content` where the match starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_offset: Option<u32>,
    /// Optional length of the matched substring.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_length: Option<u32>,
}

impl SearchResult {
    /// Construct a new search result.
    #[must_use]
    pub fn new(
        file: impl Into<String>,
        line: u32,
        content: impl Into<String>,
        context: impl Into<String>,
        score: f32,
        match_type: MatchType,
    ) -> Self {
        Self {
            file: file.into(),
            line,
            content: content.into(),
            score: score.clamp(0.0, 1.0),
            context: context.into(),
            match_type,
            match_offset: None,
            match_length: None,
        }
    }
}

/// A bundle of search results with metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResults {
    /// The query that produced these results.
    pub query: String,
    /// The matched results, already sorted by descending score.
    pub results: Vec<SearchResult>,
    /// Number of files scanned.
    pub files_scanned: usize,
    /// Time taken to execute the search (milliseconds).
    pub duration_ms: u64,
}

impl SearchResults {
    /// Returns `true` if no results were found.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Returns the top N results.
    #[must_use]
    pub fn top(&self, n: usize) -> &[SearchResult] {
        &self.results[..self.results.len().min(n)]
    }

    /// Returns only exact matches.
    pub fn exact(&self) -> impl Iterator<Item = &SearchResult> {
        self.results
            .iter()
            .filter(|r| r.match_type == MatchType::Exact)
    }

    /// Returns only semantic matches.
    pub fn semantic(&self) -> impl Iterator<Item = &SearchResult> {
        self.results
            .iter()
            .filter(|r| r.match_type == MatchType::Semantic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(file: &str, line: u32, score: f32, ty: MatchType) -> SearchResult {
        SearchResult::new(file, line, "x", "ctx", score, ty)
    }

    #[test]
    fn match_type_weights_are_ordered() {
        assert!(MatchType::Exact.weight() > MatchType::Semantic.weight());
        assert!(MatchType::Semantic.weight() > MatchType::Structural.weight());
        assert!(MatchType::Structural.weight() > MatchType::Behavioral.weight());
    }

    #[test]
    fn score_is_clamped() {
        let r = SearchResult::new("f", 1, "c", "ctx", 1.5, MatchType::Exact);
        assert_eq!(r.score, 1.0);
        let r = SearchResult::new("f", 1, "c", "ctx", -0.1, MatchType::Exact);
        assert_eq!(r.score, 0.0);
    }

    #[test]
    fn top_n_truncates_correctly() {
        let r = SearchResults {
            query: "x".into(),
            results: vec![
                sample("a", 1, 0.9, MatchType::Exact),
                sample("b", 2, 0.7, MatchType::Semantic),
                sample("c", 3, 0.5, MatchType::Structural),
            ],
            files_scanned: 3,
            duration_ms: 5,
        };
        assert_eq!(r.top(2).len(), 2);
        assert_eq!(r.top(10).len(), 3);
    }
}
