//! Codebase scanning for pattern matches.
//!
//! The scanner walks a directory tree, filters files by language and glob
//! patterns, and runs the [`PatternMatcher`](crate::pattern::PatternMatcher)
//! against each file. It supports parallel file processing via the
//! [`parallel`] sub-module and configurable include/exclude rules via
//! [`walker`].

pub mod parallel;
pub mod walker;

pub use walker::FileWalker;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

use crate::language::LanguageAdapter;
use crate::pattern::matcher::PatternMatcher;
use crate::pattern::Pattern;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for a codebase scan.
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Root directory to scan.
    pub target_dir: PathBuf,
    /// Glob patterns for files to *include* (empty = include all).
    pub include_patterns: Vec<String>,
    /// Glob patterns for files to *exclude*.
    pub exclude_patterns: Vec<String>,
    /// Whether to respect `.gitignore` rules.
    pub respect_gitignore: bool,
    /// Maximum file size in bytes (files larger than this are skipped).
    pub max_file_size: usize,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            target_dir: PathBuf::from("."),
            include_patterns: vec![],
            exclude_patterns: vec![],
            respect_gitignore: true,
            max_file_size: 1_000_000, // 1 MB
        }
    }
}

// ---------------------------------------------------------------------------
// Scan results
// ---------------------------------------------------------------------------

/// Aggregated result of scanning a codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Total number of files scanned.
    pub total_files_scanned: usize,
    /// Total number of pattern matches found across all files.
    pub total_matches: usize,
    /// Individual matches.
    pub matches: Vec<ScanMatch>,
    /// Wall-clock duration of the scan in milliseconds.
    pub duration_ms: u64,
}

/// A single match found during scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMatch {
    /// File in which the match was found.
    pub file_path: PathBuf,
    /// 1-indexed line number.
    pub line: usize,
    /// 0-indexed column (byte offset within the line).
    pub column: usize,
    /// The matched source text.
    pub matched_text: String,
    /// A few lines of context *before* the match.
    pub context_before: String,
    /// A few lines of context *after* the match.
    pub context_after: String,
}

// ---------------------------------------------------------------------------
// Scanner
// ---------------------------------------------------------------------------

/// Main scanner that orchestrates file walking and pattern matching.
pub struct Scanner {
    config: ScanConfig,
    language: Box<dyn LanguageAdapter>,
}

impl Scanner {
    /// Create a new scanner with the given configuration and language adapter.
    pub fn new(config: ScanConfig, language: Box<dyn LanguageAdapter>) -> Self {
        Self { config, language }
    }

    /// Scan the target directory for pattern matches.
    ///
    /// Files are filtered by extension (via the language adapter) and the
    /// configured include/exclude globs. Each eligible file is parsed and
    /// matched against the pattern.
    ///
    /// # Errors
    ///
    /// Returns [`CodemodError::Scan`] if the target directory cannot be
    /// walked or a file cannot be read.
    pub fn scan(&self, pattern: &Pattern) -> crate::Result<ScanResult> {
        let start = Instant::now();

        // 1. Collect eligible files.
        let walker = FileWalker::new(&self.config)?;
        let files = walker.collect_files(&*self.language)?;

        log::info!("Found {} eligible files to scan", files.len());

        // 2. Scan files (sequentially; for parallel see `parallel` module).
        let matcher = PatternMatcher::new(self.make_language_clone());
        let mut scan_matches = Vec::new();
        let mut total_files_scanned: usize = 0;

        for file_path in &files {
            let source = match std::fs::read_to_string(file_path) {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("Skipping {}: {e}", file_path.display());
                    continue;
                }
            };

            total_files_scanned += 1;

            match matcher.find_matches(&source, pattern) {
                Ok(matches) => {
                    for m in matches {
                        let (ctx_before, ctx_after) =
                            Self::extract_context(&source, m.start_position.line, 3);
                        scan_matches.push(ScanMatch {
                            file_path: file_path.clone(),
                            line: m.start_position.line + 1, // 1-indexed
                            column: m.start_position.column,
                            matched_text: m.matched_text.clone(),
                            context_before: ctx_before,
                            context_after: ctx_after,
                        });
                    }
                }
                Err(e) => {
                    log::warn!("Error matching in {}: {e}", file_path.display());
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        let total_matches = scan_matches.len();

        Ok(ScanResult {
            total_files_scanned,
            total_matches,
            matches: scan_matches,
            duration_ms,
        })
    }

    // -----------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------

    /// Extract a few lines of context around a given line number.
    fn extract_context(source: &str, line: usize, radius: usize) -> (String, String) {
        let lines: Vec<&str> = source.lines().collect();
        let start_before = line.saturating_sub(radius);
        let end_after = (line + radius + 1).min(lines.len());

        let before = lines[start_before..line].join("\n");
        let after = if line + 1 < lines.len() {
            lines[(line + 1)..end_after].join("\n")
        } else {
            String::new()
        };

        (before, after)
    }

    /// Create a boxed clone of the language adapter.
    ///
    /// Because `LanguageAdapter` is object-safe but not `Clone`, we capture
    /// the data returned by the trait methods into a small owned struct.
    fn make_language_clone(&self) -> Box<dyn LanguageAdapter> {
        Box::new(StaticLanguageInfo::snapshot(&*self.language))
    }
}

// ---------------------------------------------------------------------------
// StaticLanguageInfo — owned snapshot of a LanguageAdapter
// ---------------------------------------------------------------------------

/// A small, owned snapshot of a [`LanguageAdapter`] that can be cheaply moved
/// into a new [`PatternMatcher`].
///
/// This exists because `Box<dyn LanguageAdapter>` is not `Clone`. The snapshot
/// captures all returned slices and the `Language` value so that a new
/// `PatternMatcher` can be constructed without requiring `Arc`.
pub(crate) struct StaticLanguageInfo {
    name: String,
    lang: tree_sitter::Language,
    extensions: Vec<String>,
    statements: Vec<String>,
    expressions: Vec<String>,
    identifiers: Vec<String>,
}

impl StaticLanguageInfo {
    pub(crate) fn snapshot(adapter: &dyn LanguageAdapter) -> Self {
        Self {
            name: adapter.name().to_string(),
            lang: adapter.language(),
            extensions: adapter
                .file_extensions()
                .iter()
                .map(|s| s.to_string())
                .collect(),
            statements: adapter
                .statement_node_types()
                .iter()
                .map(|s| s.to_string())
                .collect(),
            expressions: adapter
                .expression_node_types()
                .iter()
                .map(|s| s.to_string())
                .collect(),
            identifiers: adapter
                .identifier_node_types()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

impl LanguageAdapter for StaticLanguageInfo {
    fn name(&self) -> &str {
        &self.name
    }

    fn language(&self) -> tree_sitter::Language {
        self.lang.clone()
    }

    fn file_extensions(&self) -> &[&str] {
        // Leak a small slice for the lifetime of the program. This is bounded
        // because StaticLanguageInfo is created at most a handful of times per
        // scan invocation.
        let refs: Vec<&str> = self.extensions.iter().map(|s| s.as_str()).collect();
        Box::leak(refs.into_boxed_slice())
    }

    fn statement_node_types(&self) -> &[&str] {
        let refs: Vec<&str> = self.statements.iter().map(|s| s.as_str()).collect();
        Box::leak(refs.into_boxed_slice())
    }

    fn expression_node_types(&self) -> &[&str] {
        let refs: Vec<&str> = self.expressions.iter().map(|s| s.as_str()).collect();
        Box::leak(refs.into_boxed_slice())
    }

    fn identifier_node_types(&self) -> &[&str] {
        let refs: Vec<&str> = self.identifiers.iter().map(|s| s.as_str()).collect();
        Box::leak(refs.into_boxed_slice())
    }
}
