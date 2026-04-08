//! Parallel file scanning using Rayon.
//!
//! When scanning large codebases the I/O and parsing work can be distributed
//! across multiple threads. This module provides a thin wrapper around
//! [`rayon`] that parallelizes the per-file matching step.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rayon::prelude::*;

use crate::language::LanguageAdapter;
use crate::pattern::matcher::PatternMatcher;
use crate::pattern::Pattern;
use crate::scanner::{ScanMatch, StaticLanguageInfo};

/// Result of scanning a single file in parallel.
#[derive(Debug)]
pub struct FileResult {
    /// The file that was scanned.
    pub file_path: PathBuf,
    /// Matches found in this file.
    pub matches: Vec<ScanMatch>,
    /// Error message if scanning failed for this file.
    pub error: Option<String>,
}

/// Scan a batch of files in parallel using Rayon's thread pool.
///
/// Each file is read, parsed, and matched independently. Results are
/// collected into a `Vec<FileResult>`.
///
/// # Arguments
///
/// - `files`: Paths to the files to scan.
/// - `pattern`: The transformation pattern to match against.
/// - `language`: The language adapter (must be `Send + Sync`).
///
/// # Returns
///
/// A vector of [`FileResult`]s, one per input file (in arbitrary order).
pub fn scan_files_parallel(
    files: &[PathBuf],
    pattern: &Pattern,
    language: &dyn LanguageAdapter,
) -> Vec<FileResult> {
    // Capture a snapshot of the language adapter data so each Rayon task can
    // build its own PatternMatcher without needing Send + Sync on the
    // original adapter.
    let snapshot = StaticLanguageInfo::snapshot(language);
    let lang_name = snapshot.name().to_string();
    let lang_obj = snapshot.language();
    let lang_exts: Vec<String> = snapshot
        .file_extensions()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let lang_stmts: Vec<String> = snapshot
        .statement_node_types()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let lang_exprs: Vec<String> = snapshot
        .expression_node_types()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let lang_ids: Vec<String> = snapshot
        .identifier_node_types()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let pattern = pattern.clone();
    let results: Arc<Mutex<Vec<FileResult>>> =
        Arc::new(Mutex::new(Vec::with_capacity(files.len())));

    files.par_iter().for_each(|file_path| {
        let file_result = scan_single_file(
            file_path,
            &pattern,
            &lang_name,
            &lang_obj,
            &lang_exts,
            &lang_stmts,
            &lang_exprs,
            &lang_ids,
        );
        results.lock().unwrap().push(file_result);
    });

    Arc::try_unwrap(results).unwrap().into_inner().unwrap()
}

/// Scan a single file (called from within a Rayon task).
#[allow(clippy::too_many_arguments)]
fn scan_single_file(
    file_path: &PathBuf,
    pattern: &Pattern,
    lang_name: &str,
    lang_obj: &tree_sitter::Language,
    lang_exts: &[String],
    lang_stmts: &[String],
    lang_exprs: &[String],
    lang_ids: &[String],
) -> FileResult {
    // Read file.
    let source = match std::fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            return FileResult {
                file_path: file_path.clone(),
                matches: Vec::new(),
                error: Some(format!("Failed to read file: {e}")),
            };
        }
    };

    // Build a per-thread language adapter.
    let adapter = InlineLanguageAdapter {
        name: lang_name.to_string(),
        lang: lang_obj.clone(),
        extensions: lang_exts.to_vec(),
        statements: lang_stmts.to_vec(),
        expressions: lang_exprs.to_vec(),
        identifiers: lang_ids.to_vec(),
    };

    let matcher = PatternMatcher::new(Box::new(adapter));

    match matcher.find_matches(&source, pattern) {
        Ok(matches) => {
            let scan_matches: Vec<ScanMatch> = matches
                .into_iter()
                .map(|m| {
                    let (ctx_before, ctx_after) =
                        extract_context(&source, m.start_position.line, 3);
                    ScanMatch {
                        file_path: file_path.clone(),
                        line: m.start_position.line + 1,
                        column: m.start_position.column,
                        matched_text: m.matched_text,
                        context_before: ctx_before,
                        context_after: ctx_after,
                    }
                })
                .collect();
            FileResult {
                file_path: file_path.clone(),
                matches: scan_matches,
                error: None,
            }
        }
        Err(e) => FileResult {
            file_path: file_path.clone(),
            matches: Vec::new(),
            error: Some(format!("Matching error: {e}")),
        },
    }
}

/// Extract context lines around a given 0-indexed line number.
fn extract_context(source: &str, line: usize, radius: usize) -> (String, String) {
    let lines: Vec<&str> = source.lines().collect();
    let start = line.saturating_sub(radius);
    let end = (line + radius + 1).min(lines.len());

    let before = lines[start..line].join("\n");
    let after = if line + 1 < lines.len() {
        lines[(line + 1)..end].join("\n")
    } else {
        String::new()
    };

    (before, after)
}

// ---------------------------------------------------------------------------
// Inline language adapter for parallel tasks
// ---------------------------------------------------------------------------

/// A small owned language adapter used inside Rayon tasks.
struct InlineLanguageAdapter {
    name: String,
    lang: tree_sitter::Language,
    extensions: Vec<String>,
    statements: Vec<String>,
    expressions: Vec<String>,
    identifiers: Vec<String>,
}

unsafe impl Send for InlineLanguageAdapter {}
unsafe impl Sync for InlineLanguageAdapter {}

impl LanguageAdapter for InlineLanguageAdapter {
    fn name(&self) -> &str {
        &self.name
    }
    fn language(&self) -> tree_sitter::Language {
        self.lang.clone()
    }
    fn file_extensions(&self) -> &[&str] {
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
